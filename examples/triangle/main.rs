
use three_d::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let screenshot_path = if args.len() > 1 { Some(args[1].clone()) } else {None};

    let window = Window::new("Triangle", Some((1280, 720))).unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(&context, vec3(0.0, 0.0, 2.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0),
                                             degrees(45.0), window.viewport().aspect(), 0.1, 10.0);

    // Create a colored triangle
    let positions: Vec<f32> = vec![
        0.5, -0.5, 0.0, // bottom right
        -0.5, -0.5, 0.0,// bottom left
        0.0,  0.5, 0.0 // top
    ];
    let colors: Vec<u8> = vec![
        255, 0, 0, 255,   // bottom right
        0, 255, 0, 255,   // bottom left
        0, 0, 255, 255   // top
    ];
    let cpu_mesh = CPUMesh {
        positions, colors: Some(colors), ..Default::default()
    };
    let mesh = Mesh::new(&context, &cpu_mesh).unwrap();

    // main loop
    window.render_loop(move |frame_input|
    {
        camera.set_aspect(frame_input.viewport.aspect());
        Screen::write(&context, &ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0), || {
            let transformation = Mat4::from_angle_y(radians((frame_input.accumulated_time * 0.005) as f32));
            mesh.render_color(RenderStates::default(), frame_input.viewport, &transformation, &camera)?;
            Ok(())
        }).unwrap();

        #[cfg(target_arch = "x86_64")]
        if let Some(ref path) = screenshot_path {
            use three_d::io::*;
            let pixels = Screen::read_color(&context, frame_input.viewport).unwrap();
            Saver::save_pixels(path, &pixels, frame_input.viewport.width, frame_input.viewport.height).unwrap();
            std::process::exit(1);
        }
    }).unwrap();
}