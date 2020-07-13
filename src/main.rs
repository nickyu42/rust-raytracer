extern crate cgmath;
extern crate image;

mod color;

use cgmath::prelude::*;
use cgmath::{Point3, Vector3};
use image::{DynamicImage, GenericImage};

use color::Color;

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f64>,
    pub direction: Vector3<f64>,
}

impl Ray {
    /// Creates a camera ray
    pub fn create_prime(x: u32, y: u32, scene: &Scene) -> Ray {
        // Adjust fov, essentially is a ratio of x with respect to y
        let fov_adjustment = (scene.fov.to_radians() / 2.0).tan();
        let sensor_x = (((x as f64 + 0.5) / scene.width as f64) * 2.0 - 1.0) * scene.aspect_ratio * fov_adjustment;
        let sensor_y = (1.0 - ((y as f64 + 0.5) / scene.height as f64) * 2.0) * fov_adjustment;

        Ray {
            origin: Point3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(sensor_x, sensor_y, -1.0).normalize(),
        }
    }
}

#[derive(Debug)]
struct Collision<'a> {
    distance: f64,
    object: &'a Sphere,
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<f64>;
    fn surface_normal(&self, hit_point: &Point3<f64>) -> Vector3<f64>;
}

#[derive(Debug)]
pub struct Sphere {
    pub center: Point3<f64>,
    pub radius: f64,
    pub color: Color,
    pub albedo: f32,
    pub ks: f32,
    pub kd: f32,
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        // Vector from the sphere center to ray
        let hypo: Vector3<f64> = self.center - ray.origin;

        // Calculate the length of the adjacent side of the triangle
        let adj = hypo.dot(ray.direction);

        // Calculate the orthogonal distance from sphere origin to ray
        let d = hypo.dot(hypo) - (adj * adj);

        let radius_sq = self.radius * self.radius;

        if d > radius_sq {
            return None;
        }

        let thickness = (radius_sq - d).sqrt();

        let t0 = adj - thickness;
        let t1 = adj + thickness;

        if t0 < 0.0 && t1 < 0.0 {
            return None;
        }

        Some(if t0 < t1 { t0 } else { t1 })
    }

    fn surface_normal(&self, hit_point: &Point3<f64>) -> Vector3<f64> {
        (hit_point - self.center).normalize()
    }
}

// pub struct DirectionalLight {
//     pub direction: Vector3<f64>,
//     pub color: Color,
//     pub intensity: f32,
// }
//
// pub struct SphereLight {
//     pub position: Point3<f64>,
//     pub color: Color,
//     pub intensity: f32,
// }

pub enum Light {
    DirectionalLight {
        direction: Vector3<f64>,
        color: Color,
        intensity: f32,
    },
    SphereLight {
        position: Point3<f64>,
        color: Color,
        intensity: f32,
    },
}

impl Light {
    fn color(&self) -> &Color {
        match self {
            Light::DirectionalLight { direction: _, color, intensity: _ } => color,
            Light::SphereLight { position: _, color, intensity: _ } => color
        }
    }
}
//
//     fn intensity(&self) -> f32 {
//         match self {
//             Light::DirectionalLight => self.intensity,
//             Light::SphereLight => self.intensity
//         }
//     }
// }

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub aspect_ratio: f64,
    pub objects: Vec<Sphere>,
    pub lights: Vec<Light>,
    pub shadow_bias: f64,
}

impl Scene {
    fn new(width: u32, height: u32, fov: f64, objects: Vec<Sphere>, lights: Vec<Light>, shadow_bias: f64) -> Scene {
        Scene {
            width,
            height,
            fov,
            aspect_ratio: (width as f64) / (height as f64),
            objects,
            lights,
            shadow_bias,
        }
    }

    fn trace(&self, ray: &Ray) -> Option<Collision> {
        self.objects
            .iter()
            .filter_map(|o| o.intersect(ray).map(|d| Collision { distance: d, object: o }))
            .min_by(|x: &Collision, y: &Collision| x.distance.partial_cmp(&y.distance).unwrap())
    }
}

fn get_light(light: &Light, scene: &Scene, hit_point: &Point3<f64>) -> (f32, Vector3<f64>) {
    match light {
        Light::DirectionalLight { direction, color: _, intensity } => {
            let direction_to_light = -*direction;

            let shadow_ray = Ray {
                origin: hit_point + (direction_to_light * scene.shadow_bias),
                direction: direction_to_light,
            };
            let in_light = scene.trace(&shadow_ray).is_none();

            let light_intensity = if in_light { *intensity } else { 0.0 };

            (light_intensity, direction_to_light)
        },
        Light::SphereLight { position, color: _, intensity } => {
            let direction_to_light = (position - hit_point).normalize();

            let shadow_ray = Ray {
                origin: hit_point + (direction_to_light * scene.shadow_bias),
                direction: direction_to_light,
            };

            let shadow_intersect = scene.trace(&shadow_ray);
            let in_light = shadow_intersect.is_none() || shadow_intersect.unwrap().distance > (position - hit_point).magnitude();

            let light_intensity = if in_light {
                // TODO: create var for distance from light position to hit_point
                let distance_to_light_sq = (position - hit_point).magnitude2();
                intensity / (4.0 * std::f64::consts::PI * distance_to_light_sq) as f32
            } else {
                0.0
            };
            // let distance_to_light_sq = (position - hit_point).magnitude2();
            // let light_intensity = intensity / (4.0 * std::f64::consts::PI * distance_to_light_sq) as f32;

            (light_intensity, direction_to_light)
        },
    }
}

fn get_color(scene: &Scene, ray: &Ray) -> Color {
    let mut color = Color {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    let intersection = scene.trace(&ray);

    if intersection.is_some() {
        let coll = intersection.unwrap();
        let hit_point = ray.origin + (ray.direction * coll.distance);
        let surface_normal = coll.object.surface_normal(&hit_point);

        for light in &scene.lights {
            let (light_intensity, direction_to_light) = get_light(light, scene, &hit_point);

            // Compute diffuse component without a K_d
            let diffuse = (surface_normal.dot(direction_to_light) as f32).max(0.0) * light_intensity;

            // Compute specular component using Blinn-Phong
            let view_vector = -ray.direction.normalize();
            let half_vector = (direction_to_light + view_vector).normalize();
            let specular = surface_normal.dot(half_vector).powi(25) as f32 * coll.object.kd * light_intensity;

            let light_reflected = coll.object.albedo / std::f32::consts::PI;
            let light_color = light.color().clone() * (diffuse * coll.object.ks) * light_reflected;

            color = color + coll.object.color.clone() * light_color + specular
        }
    }

    color.clamp();
    color
}

fn render(scene: &Scene) -> DynamicImage {
    let mut image = DynamicImage::new_rgb8(scene.width, scene.height);

    for x in 0..scene.width {
        for y in 0..scene.height {
            let ray = Ray::create_prime(x, y, scene);

            let color = get_color(scene, &ray);

            image.put_pixel(x, y, color.to_rgba());
        }
    }
    image
}

fn main() {
    // let mut rng = rand::thread_rng();

    // let spheres: Vec<Sphere> =
    //     (0..20)
    //         .map(|_| Sphere {
    //             center: Point3::new(rng.gen::<f64>() * 30.0 - 15.0, rng.gen::<f64>() * 30.0 - 15.0, rng.gen::<f64>() * -20.0 - 10.0),
    //             radius: 1.0 + rng.gen::<f64>() * 3.0,
    //             color: Color {
    //                 red: rng.gen(),
    //                 green: rng.gen(),
    //                 blue: rng.gen(),
    //             },
    //             albedo: rng.gen(),
    //         }).collect();

    let scene = Scene::new(800, 800, 90.0, vec![
        Sphere {
            center: Point3::new(0.0, -2.5, -5.0),
            radius: 1.0,
            color: Color {
                red: 0.4,
                green: 1.0,
                blue: 0.4,
            },
            albedo: 0.5,
            ks: 0.5,
            kd: 0.05,
        },
        Sphere {
            center: Point3::new(0.0, 0.0, -5.0),
            radius: 1.0,
            color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.4,
            },
            albedo: 0.5,
            ks: 0.5,
            kd: 0.05,
        },
        Sphere {
            center: Point3::new(3.0, 0.0, -5.0),
            radius: 2.0,
            color: Color {
                red: 0.4,
                green: 0.3,
                blue: 1.0,
            },
            albedo: 0.5,
            ks: 0.5,
            kd: 0.05,
        },
    ], vec![
        Light::DirectionalLight {
            direction: Vector3::new(-1.0, -1.0, -1.0).normalize(),
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
            },
            intensity: 10.0,
        },
        Light::DirectionalLight {
            direction: Vector3::new(0.0, 1.0, 0.0).normalize(),
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
            },
            intensity: 5.0,
        },
        Light::DirectionalLight {
            direction: Vector3::new(0.0, -0.3, 1.0).normalize(),
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
            },
            intensity: 1.0,
        },
        Light::SphereLight {
            position: Point3::new(-1.2, 0.0, -4.5),
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0
            },
            intensity: 30.0
        }
    ], 1e-13);
    render(&scene).save("test.png").unwrap();
}
