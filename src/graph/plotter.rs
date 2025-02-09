use plotters::chart::ChartBuilder;
use plotters::prelude::{BitMapBackend, Circle, IntoDrawingArea};
use plotters::series::LineSeries;
use plotters::style::{ShapeStyle, BLUE, RED, WHITE};

use crate::graph::graph::Node;

pub(super) struct Plotter {
    all_nodes: Vec<Node>,
    color_nodes: Vec<Node>,
    x_y_range: std::ops::Range<f32>,
}

impl Plotter {
    pub(super) fn new(x_y_range: std::ops::Range<f32>) -> Self {
        Plotter {
            all_nodes: vec![],
            color_nodes: vec![],
            x_y_range: x_y_range,
        }
    }

    pub(super) fn set_connected_nodes(&mut self, nodes: &Vec<Node>) {
        self.all_nodes = nodes.clone();
    }

    pub(super) fn set_isolated_nodes(&mut self, nodes: &Vec<Node>) {
        self.color_nodes = nodes.clone();
    }

    pub(super) fn plot(
        &self,
        file_name: &str,
        title: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(file_name, (1024, 1024)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        // Define the chart and the axes
        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(self.x_y_range.clone(), self.x_y_range.clone())?; // Adjust the ranges as needed

        chart.configure_mesh().draw()?;

        // Add points to connect
        for i in 0..self.all_nodes.len() {
            let point_1 = &self.all_nodes[i].vector;
            for connected_node in self.all_nodes[i].connected.iter() {
                let point_2 = &self.all_nodes[*connected_node as usize].vector;
                let connected_points = vec![(point_1[0], point_1[1]), (point_2[0], point_2[1])];
                chart.draw_series(LineSeries::new(connected_points, RED))?;
            }
        }

        // Add isolated points
        let color_points = self.color_nodes.iter().map(|node| {
            Circle::new(
                (node.vector[0], node.vector[1]),
                5,
                ShapeStyle::from(&BLUE).filled(),
            )
        });
        chart.draw_series(color_points)?;

        root.present()?;

        Ok(())
    }
}
