use gtk4::prelude::*;
use plotters::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea};
//use plotters_cairo::CairoBackend;
//use gtk4::cairo::{ ImageSurface, Format, Context };

fn main() {
    let app = Application::builder().build();
    
    app.connect_activate(build_gui);
    app.run();
}

fn build_gui(app: &Application){
    let win = ApplicationWindow::builder().application(app).default_width(1024).default_height(768).title("Test").build();
    let drawing_area: DrawingArea = DrawingArea::builder().build();

    drawing_area.set_draw_func(|_drawing_area, cr, width, height| {
        // --- 🎨 Custom Drawing Logic Starts Here ---

        let root = plotters_cairo::CairoBackend::new(&cr, (1024,768)).unwrap().into_drawing_area();
        let _ = root.fill(&WHITE);

        let root = root.margin(10, 10, 10, 10);
        // After this point, we should be able to construct a chart context
        let mut chart = ChartBuilder::on(&root)
            // Set the caption of the chart
            .caption("This is our first plot", ("sans-serif", 40).into_font())
            // Set the size of the label region
            .x_label_area_size(20)
            .y_label_area_size(40)
            // Finally attach a coordinate on the drawing area and make a chart context
            .build_cartesian_2d(0f32..10f32, 0f32..10f32).unwrap();

        // Then we can draw a mesh
        let _ = chart
            .configure_mesh()
            // We can customize the maximum number of labels allowed for each axis
            .x_labels(5)
            .y_labels(5)
            // We can also change the format of the label text
            .y_label_formatter(&|x| format!("{:.3}", x))
            .draw();

        // And we can draw something in the drawing area
        let _ = chart.draw_series(LineSeries::new(
            vec![(0.0, 0.0), (5.0, 5.0), (8.0, 7.0)],
            &RED,
        ));
        // Similarly, we can draw point series
        let _ = chart.draw_series(PointSeries::of_element(
            vec![(0.0, 0.0), (5.0, 5.0), (8.0, 7.0)],
            5,
            &RED,
            &|c, s, st| {
                return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
            },
        ));
        
        let _ = root.present();
        // --- Custom Drawing Logic Ends Here ---
    });

    win.set_child(Some(&drawing_area));
    win.present();
}
