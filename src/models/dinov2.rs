use crate::{ops, DataLoader, Metric, MinOptMax, Options, OrtEngine};
use anyhow::Result;
use image::DynamicImage;
use ndarray::{Array, IxDyn};
use std::path::PathBuf;
use usearch::ffi::{IndexOptions, MetricKind, ScalarKind};

#[derive(Debug)]
pub enum Model {
    S,
    B,
}

#[derive(Debug)]
pub struct Dinov2 {
    engine: OrtEngine,
    pub height: MinOptMax,
    pub width: MinOptMax,
    pub batch: MinOptMax,
    pub hidden_size: usize,
}

impl Dinov2 {
    pub fn new(options: &Options) -> Result<Self> {
        let engine = OrtEngine::new(options)?;
        let (batch, height, width) = (
            engine.inputs_minoptmax()[0][0].to_owned(),
            engine.inputs_minoptmax()[0][2].to_owned(),
            engine.inputs_minoptmax()[0][3].to_owned(),
        );
        let which = match &options.onnx_path {
            s if s.contains("b14") => Model::B,
            s if s.contains("s14") => Model::S,
            _ => todo!(),
        };
        let hidden_size = match which {
            Model::S => 384,
            Model::B => 768,
        };
        engine.dry_run()?;

        Ok(Self {
            engine,
            height,
            width,
            batch,
            hidden_size,
        })
    }

    pub fn run(&mut self, xs: &[DynamicImage]) -> Result<Array<f32, IxDyn>> {
        let xs_ = ops::resize(xs, self.height.opt as u32, self.width.opt as u32, true)?;
        let ys: Vec<Array<f32, IxDyn>> = self.engine.run(&[xs_])?;
        let ys = ys[0].to_owned();
        let ys = ops::norm(&ys);
        Ok(ys)
    }

    pub fn build_index(&self, metric: Metric) -> Result<usearch::Index> {
        let metric = match metric {
            Metric::IP => MetricKind::IP,
            Metric::L2 => MetricKind::L2sq,
            Metric::Cos => MetricKind::Cos,
        };
        let options = IndexOptions {
            metric,
            dimensions: self.hidden_size,
            quantization: ScalarKind::F16,
            ..Default::default()
        };
        Ok(usearch::new_index(&options)?)
    }

    pub fn query_from_folder(
        &mut self,
        qurey: &str,
        gallery: &str,
        metric: Metric,
    ) -> Result<Vec<(usize, f32, PathBuf)>> {
        // load query
        let query = DataLoader::try_read(qurey)?;
        let query = self.run(&[query])?;

        // build index & gallery
        let index = self.build_index(metric)?;
        let dl = DataLoader::default()
            .with_batch(self.batch.opt as usize)
            .load(gallery)?;
        let paths = dl.paths().to_owned();
        index.reserve(paths.len())?;

        // load feats
        for (idx, (x, _path)) in dl.enumerate() {
            let y = self.run(&x)?;
            index.add(idx as u64, &y.into_raw_vec())?;
        }

        // output
        let matches = index.search(&query.into_raw_vec(), index.size())?;
        let mut results: Vec<(usize, f32, PathBuf)> = Vec::new();
        matches
            .keys
            .into_iter()
            .zip(matches.distances)
            .for_each(|(k, score)| {
                results.push((k as usize, score, paths[k as usize].to_owned()));
            });

        Ok(results)
    }

    pub fn query_from_vec(
        &mut self,
        qurey: &str,
        gallery: &[&str],
        metric: Metric,
    ) -> Result<Vec<(usize, f32, PathBuf)>> {
        // load query
        let query = DataLoader::try_read(qurey)?;
        let query = self.run(&[query])?;

        // build index & gallery
        let index = self.build_index(metric)?;
        index.reserve(gallery.len())?;
        let mut dl = DataLoader::default().with_batch(self.batch.opt as usize);
        gallery.iter().for_each(|x| {
            dl.load(x).unwrap();
        });

        // load feats
        let paths = dl.paths().to_owned();
        for (idx, (x, _path)) in dl.enumerate() {
            let y = self.run(&x)?;
            index.add(idx as u64, &y.into_raw_vec())?;
        }

        // output
        let matches = index.search(&query.into_raw_vec(), index.size())?;
        let mut results: Vec<(usize, f32, PathBuf)> = Vec::new();
        matches
            .keys
            .into_iter()
            .zip(matches.distances)
            .for_each(|(k, score)| {
                results.push((k as usize, score, paths[k as usize].to_owned()));
            });

        Ok(results)
    }
}
