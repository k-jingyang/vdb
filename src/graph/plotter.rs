use plotters::chart::ChartBuilder;
use plotters::prelude::{BitMapBackend, Circle, IntoDrawingArea};
use plotters::series::LineSeries;
use plotters::style::{ShapeStyle, BLUE, RED, WHITE};

use crate::constant::PLOT_GRAPH_X_Y_RANGE;
use crate::graph::graph::Node;

pub(super) struct Plotter {
    all_nodes: Vec<Node>,
    color_nodes: Vec<Node>,
}

impl Plotter {
    pub(super) fn new() -> Self {
        Plotter {
            all_nodes: vec![],
            color_nodes: vec![],
        }
    }

    pub(super) fn set_connected_nodes(&mut self, nodes: &Vec<Node>) {
        self.all_nodes = nodes.clone();
    }

    pub(super) fn set_isolated_nodes(&mut self, nodes: &Vec<Node>) {
        self.color_nodes = nodes.clone();
    }

    pub(super) fn plot(&self, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(file_name, (1024, 1024)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        // Define the chart and the axes
        let mut chart = ChartBuilder::on(&root)
            .caption("Vectors", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(PLOT_GRAPH_X_Y_RANGE.0, PLOT_GRAPH_X_Y_RANGE.1)?; // Adjust the ranges as needed

        chart.configure_mesh().draw()?;

        // Add points to connect
        for i in 0..self.all_nodes.len() {
            let point_1 = self.all_nodes[i].vector;
            for connected_node in self.all_nodes[i].connected.iter() {
                let point_2 = self.all_nodes[*connected_node].vector;
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
