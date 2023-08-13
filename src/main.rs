//extern crate kiss3d;
//extern crate nalgebra as na;

use std::collections::HashSet;
use std::io::Cursor;
use std::ops::Index;

use kiss3d::camera::{ArcBall, Camera};
use kiss3d::event::{MouseButton, WindowEvent};
use kiss3d::light::Light;
use kiss3d::nalgebra::{Const, OPoint, Point, Point2, Point3, UnitQuaternion, Vector2, Vector3};
use kiss3d::ncollide3d::math::Translation;
use kiss3d::ncollide3d::procedural::{IndexBuffer, TriMesh};
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::renderer::Renderer;
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};
use log::info;
use self_cell::self_cell;
use stl_io::Vector;
use triangles::prelude::Plane3d;
use triangles::prelude::ReferencedTriangle;
use triangles::prelude::{
    IndexedTriangleList, Point3d, StaticLine3d, Triangle3d, TriangleTopology, Vector3d,
};
use wasm_bindgen::prelude::*;

type TopologyType<'a> = TriangleTopology<'a, Vector3d>;

self_cell!(
    struct GeometryData {
        owner: IndexedTriangleList<Vector3d>,
        #[covariant]
        dependent: TopologyType,
    }
);

struct AppState {
    data: GeometryData,
    camera: ArcBall,
    rot: UnitQuaternion<f32>,
    last_pos: Point2<f32>,
    group: SceneNode,
    hit_marker: Option<SceneNode>,
    selection_state: Option<usize>,
}

impl State for AppState {
    fn step(&mut self, window: &mut Window) {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Pos(_, _) => {}
                WindowEvent::Size(_, _) => {}
                WindowEvent::Close => {}
                WindowEvent::Refresh => {}
                WindowEvent::Focus(_) => {}
                WindowEvent::Iconify(_) => {}
                WindowEvent::FramebufferSize(_, _) => {}
                WindowEvent::MouseButton(button, action, modif) => {
                    //info!("mouse press event on {:?} with {:?}", button, modif);
                    if button == MouseButton::Button2 {
                        let hit_point =
                            Self::find_element_at(window, &self.last_pos, &self.data, &self.camera);

                        if let Some(old_marker) = &mut self.hit_marker {
                            window.remove_node(old_marker);
                        }
                        self.hit_marker = hit_point.map(|hit| {
                            let p = hit.0;
                            let mut marker = window.add_group();
                            Self::add_sphere_at(&mut marker, &p).set_color(1.0, 0.0, 0.0);
                            //Self::add_sphere_at(&mut marker, &sight_line.p1())
                            //    .set_color(0.0, 1.0, 0.0);
                            //Self::add_sphere_at(&mut marker, &sight_line.p2())
                            //    .set_color(0.0, 0.0, 1.0);
                            /*Self::add_sphere_at(&mut marker, &webgl2triangles(&camera_pos))
                            .set_color(0.0, 1.0, 1.0);*/
                            let triangle = hit.2;
                            let [p1, p2, p3] = triangle.points();

                            let mesh = TriMesh::new(
                                vec![
                                    vector2webgl(p1.coordinates()),
                                    vector2webgl(p2.coordinates()),
                                    vector2webgl(p3.coordinates()),
                                ],
                                None,
                                None,
                                None,
                            );
                            let scale = 1.0;
                            marker.add_trimesh(mesh, Vector3::new(scale, scale, scale));
                            marker
                        });
                    }
                }
                WindowEvent::CursorPos(x, y, modifiers) => {
                    self.last_pos = Point2::new(x as f32, y as f32);

                    let hit_point =
                        Self::find_element_at(window, &self.last_pos, &self.data, &self.camera);

                    let selection_state = hit_point.map(|p| p.2.idx());

                    if selection_state != self.selection_state {
                        self.selection_state = selection_state;
                        if let Some(old_marker) = &mut self.hit_marker {
                            window.remove_node(old_marker);
                        }
                        self.hit_marker = hit_point.map(|hit| {
                            let p = hit.0;
                            let mut marker = window.add_group();
                            //Self::add_sphere_at(&mut marker, &p).set_color(1.0, 0.0, 0.0);
                            //Self::add_sphere_at(&mut marker, &sight_line.p1())
                            //    .set_color(0.0, 1.0, 0.0);
                            //Self::add_sphere_at(&mut marker, &sight_line.p2())
                            //    .set_color(0.0, 0.0, 1.0);
                            /*Self::add_sphere_at(&mut marker, &webgl2triangles(&camera_pos))
                            .set_color(0.0, 1.0, 1.0);*/
                            let triangle = hit.2;
                            let [p1, p2, p3] = triangle.points();

                            let mesh = TriMesh::new(
                                vec![
                                    vector2webgl(p1.coordinates()),
                                    vector2webgl(p2.coordinates()),
                                    vector2webgl(p3.coordinates()),
                                ],
                                None,
                                None,
                                None,
                            );
                            let scale = 1.0;
                            marker.add_trimesh(mesh, Vector3::new(scale, scale, scale));
                            marker
                        });
                    }
                }
                WindowEvent::CursorEnter(_) => {}
                WindowEvent::Scroll(_, _, _) => {}
                WindowEvent::Key(_, _, _) => {}
                WindowEvent::Char(_) => {}
                WindowEvent::CharModifiers(_, _) => {}
                WindowEvent::Touch(_, _, _, _, _) => {}
            }
        }
    }

    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn Renderer>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        //info!("cameras_and_effect_and_renderer: {:?}", self.camera.eye());
        (Some(&mut self.camera), None, None, None)
    }
}

impl AppState {
    fn add_sphere_at(mut node: &mut SceneNode, point: &Vector3d) -> SceneNode {
        let mut sphere = node.add_sphere(0.005);
        sphere.append_translation(&Translation::new(
            point.x.0 as f32,
            point.y.0 as f32,
            point.z.0 as f32,
        ));
        sphere
    }

    fn find_element_at<'a>(
        window: &Window,
        point: &Point2<f32>,
        data: &'a GeometryData,
        camera: &ArcBall,
    ) -> Option<(Vector3d, &'a Plane3d, &'a ReferencedTriangle<'a, Vector3d>)> {
        let window_size = Vector2::new(window.size()[0] as f32, window.size()[1] as f32);
        let (sel_pos, dir) = camera.unproject(point, &window_size);
        let sel_pos = webgl2triangles(&sel_pos);
        let sight_line = StaticLine3d::new(sel_pos, webgl2triangles(&dir));
        let topology = data.borrow_dependent();
        topology.find_first_intersection(&sight_line)
    }
}

#[cfg(not(debug_assertions))]
const LOG_LEVEL: log::Level = log::Level::Info;
#[cfg(debug_assertions)]
const LOG_LEVEL: log::Level = log::Level::Trace;
pub fn main() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(LOG_LEVEL));

    let mut window = Window::new("");
    window.set_background_color(1.0, 1.0, 1.0);

    let bytes = include_bytes!("Schublade - Front.stl");
    let mut cursor = Cursor::new(bytes);
    let stl_mesh = stl_io::read_stl(&mut cursor).expect("Cannot load stl file");
    let triangle_list: IndexedTriangleList<_> = stl_mesh.into();
    let triangle_list = triangle_list.transform_points(stl2triangles);
    let data = GeometryData::new(triangle_list, |d| {
        TriangleTopology::new(d).expect("Topology error")
    });

    let group = draw_geometry(&mut window, data.borrow_owner());

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);
    let camera = ArcBall::new(Point3::new(0.0, 0.5, -1.0), Point3::new(0.0, 0.0, 0.0));
    let state = AppState {
        group,
        rot,
        camera,
        last_pos: Point2::new(0.0, 0.0),
        data,
        hit_marker: None,
        selection_state: None,
    };

    window.render_loop(state);
    Ok(())
}

fn draw_geometry(window: &mut Window, triangle_list: &IndexedTriangleList<Vector3d>) -> SceneNode {
    let topology = TriangleTopology::new(triangle_list).expect("Errors in topology");
    let mesh: Vec<_> = topology
        .triangles_of_plane()
        .iter()
        .map(|(plane, triangles)| {
            //let normal = plane.normal();
            //let mut point_map = HashMap::new();
            let used_points: HashSet<_> = triangles
                .iter()
                .flat_map(|tr| tr.points())
                .map(|p| p.idx())
                .collect();
            let mut reverse_map = vec![0; triangle_list.points().len()].into_boxed_slice();
            let mut coords: Vec<Point<f32, 3>> = vec![Point3::default(); used_points.len()];
            let points = triangle_list.points();
            for (target, source) in used_points.iter().enumerate() {
                reverse_map[*source] = target;
                let p = points[*source];
                coords[target] = Point3::from([p[0].0 as f32, p[1].0 as f32, p[2].0 as f32]);
            }
            let faces: Vec<_> = triangles
                .iter()
                .map(|tr| Point3::from(tr.points().map(|p| reverse_map[p.idx()] as u32)))
                .collect();
            TriMesh::new(coords, None, None, Some(IndexBuffer::Unified(faces)))
        })
        .collect();

    let mut group = window.add_group();
    let scale = 1.0;
    for m in mesh {
        group.add_trimesh(m, Vector3::new(scale, scale, scale));
    }

    group.set_color(0.8, 0.8, 2.0);
    group
}

#[inline]
fn stl2triangles(p: &Vector<f32>) -> Vector3d {
    Vector3d::new(
        (p[0] as f64 / 1000.0).into(),
        (p[1] as f64 / 1000.0).into(),
        (p[2] as f64 / 1000.0).into(),
    )
}
#[inline]
fn webgl2triangles<P: Index<usize, Output = f32>>(p: &P) -> Vector3d {
    Vector3d::new(
        (p[0] as f64).into(),
        (p[1] as f64).into(),
        (p[2] as f64).into(),
    )
}
fn vector2webgl(p: Vector3d) -> OPoint<f32, Const<3>> {
    Vector3::new(p.x.0 as f32, p.y.0 as f32, p.z.0 as f32).into()
}

pub const WIN_W: u32 = 600;
pub const WIN_H: u32 = 420;
