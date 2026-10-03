struct Textures {
    [format("rgba32f")]
    RWTexture2D<float4> result;
    Texture2D<float4> environment;
    SamplerState sampler;
}

struct Scene {
    ConstantBuffer<Uniforms> uniforms;
    StructuredBuffer<Sphere> spheres;
    StructuredBuffer<Material> materials;
}

struct Uniforms {
    uint frame_count;
    uint accumulated_frames;
    uint num_spheres;
    float focal_length;
    float3 camera_center;
    uint focusing;
    column_major float3x3 rotation_matrix;
    float aperture;
    float focus_distance;
    float ground_plane_y;
    int ground_plane_material_index;
}

ParameterBlock<Textures> textures;
ParameterBlock<Scene> scene;

[shader("compute")]
[numthreads(8,8)]
void main(uint3 threadId: SV_DispatchThreadID) {
    uint scale = 1;

    var result = textures.result;
    var frame_count = scene.uniforms.frame_count;
    var accumulated_frames = scene.uniforms.accumulated_frames;

    uint2 texture_dimensions;
    result.GetDimensions(texture_dimensions.x, texture_dimensions.y);

    if (threadId.x >= texture_dimensions.x || threadId.y >= texture_dimensions.y) {
        return;
    }

    float aspect = float(texture_dimensions.x)/float(texture_dimensions.y);

    float viewport_height = 0.024;
    float viewport_width = viewport_height * aspect;
    uint max_bounces = 12;

    float delta_u = viewport_width / float(texture_dimensions.x/scale);
    float delta_v = viewport_height / float(texture_dimensions.y/scale);


    uint pixel_index = threadId.x/scale + threadId.y/scale * texture_dimensions.x/scale;
    Random random = Random(pixel_index^Random.hash(frame_count));

    // first get pure depth w/o any effects
    float3 depth_sample_dir = float3(-viewport_width/2 + (float(threadId.x/scale) + 0.5) * delta_u,
                                     viewport_height/2 - (float(threadId.y/scale) + 0.5) * delta_v,
                                     -scene.uniforms.focal_length);
    Ray depth_sample_ray = Ray(scene.uniforms.camera_center,mul(scene.uniforms.rotation_matrix, normalize(depth_sample_dir)));
    float depth = depth_sample_ray.get_depth();

    // offset should be between -.5 and .5 for subpixel aa
    float2 offset = float2(random.random()-0.5, random.random()-0.5);
    float aperture_radius = (scene.uniforms.focal_length / scene.uniforms.aperture) / 2;
    float3 ray_sample_focal_plane = scene.uniforms.focus_distance / scene.uniforms.focal_length * float3(
                                          -viewport_width/2 + (float(threadId.x/scale) + 0.5 + offset.x) * delta_u,
                                          viewport_height/2 - (float(threadId.y/scale) + 0.5 + offset.y) * delta_v,
                                          -scene.uniforms.focal_length);
    float2 ray_center = random.random_disk() * aperture_radius;
    float3 ray_direction = ray_sample_focal_plane - float3(ray_center,0);

    Ray offset_ray = Ray(mul(scene.uniforms.rotation_matrix,float3(ray_center,0))+scene.uniforms.camera_center,mul(scene.uniforms.rotation_matrix, normalize(ray_direction)));
    float3 color = offset_ray.color(max_bounces,random);

    if (any(isnan(color)) || any(isinf(color))) {
        return;
    }

    float3 old_color = result.Load(threadId.xy).rgb;
    float weight = 1/float(accumulated_frames+1);
    float3 color_accumulated = fma(color.rgb - old_color,weight,old_color);

    if (scene.uniforms.focusing == 1 && (pow(float(threadId.x) - float(texture_dimensions.x)/2,2) + pow(float(threadId.y) - float(texture_dimensions.y)/2,2) < 200)) {
        result.Store(threadId.xy,float4(float3(1,0,0),depth));
    } else {
        result.Store(threadId.xy,float4(color_accumulated,depth));
    }
}

struct Ray {
    float3 origin;
    float3 direction;

    float3 position(float t) {
        return origin + direction * t;
    };

    [mutating]
    float3 color(uint max_bounces, inout Random random) {
        float3 incoming_light = 0;
        float3 color = 1;
        float t = -1;
        int sphere_index;
        float3 normal;
        float3 hit_position;
        float3 emitted_light;
        float3 diffuse_dir;
        float3 specular_dir;
        Sphere hit_sphere;
        Material material;
        float temp_t;
        int material_index;

        for (uint i = 0; i < max_bounces; i++) {
            Interval t_bounds = Interval(0.00001,5000);
            t = -1;

            //check ground plane collision
            temp_t = (scene.uniforms.ground_plane_y - this.origin.y) / this.direction.y;
            if (t_bounds.surrounds(temp_t)) {
                t = temp_t;
                material_index = scene.uniforms.ground_plane_material_index;
                t_bounds.max = t;
                normal = float3(0,1,0);
                hit_position = this.position(t);
            }

            //check sphere collisions
            sphere_index = -1;
            for (int s = 0; s < scene.uniforms.num_spheres; s++) {
                temp_t = scene.spheres[s].hit(this,t_bounds);
                if (temp_t > 0) {
                    t = temp_t;
                    sphere_index = s;
                    t_bounds.max = t;
                }
            }
            if (sphere_index >= 0) {
                hit_sphere = scene.spheres[sphere_index];
                material_index = hit_sphere.material_index;
                hit_position = this.position(t);
                normal = (hit_position - hit_sphere.center) / hit_sphere.radius;
            }

            if (t > 0) {
                if (dot(this.direction,normal) >= 0) {
                    normal = normal * -1.0;
                }
                material = scene.materials[material_index];

                emitted_light = material.emission_color * material.emission_strength;
                incoming_light += emitted_light * color;

                color *= material.color;
                this.origin = hit_position + normal * 1e-3;

                diffuse_dir = normalize(normal + random.random_unit_vector());
                specular_dir = this.direction - 2 * dot(this.direction,normal) * normal;
                this.direction = normalize(lerp(diffuse_dir,specular_dir,material.smoothness));
            } else {
                incoming_light += this.sample_environment() * color;
                break;
            }
        }
        return float3(incoming_light);
    }

    float get_depth() {
        Interval t_bounds = Interval(0.00001,5000);
        float t = -1;

        //check ground plane collision
        float temp_t = (scene.uniforms.ground_plane_y - this.origin.y) / this.direction.y;
        if (t_bounds.surrounds(temp_t)) {
            t = temp_t;
            t_bounds.max = t;
        }

        //check sphere collisions
        for (int s = 0; s < scene.uniforms.num_spheres; s++) {
            temp_t = scene.spheres[s].hit(this,t_bounds);
            if (temp_t > 0) {
                t = temp_t;
                t_bounds.max = t;
            }
        }

        return t;
    }

    float3 sample_environment() {
        float yaw = atan2(this.direction.x, -this.direction.z);
        float pitch = acos(this.direction.y);

        float2 location = float2(yaw / (2 * float.getPi()) + 0.5 + 0.075, pitch/float.getPi());
        return textures.environment.SampleLevel(textures.sampler, location, 0.0).rgb;
    }
}

struct Sphere {
    float3 center;
    float radius;
    uint material_index;

    float hit(Ray ray, Interval t_bounds) {
        float3 oc = this.center - ray.origin;

        float h = dot(ray.direction,oc);
        float c = dot(oc,oc) - this.radius * this.radius;
        float square_discriminant = h*h - c;

        if (square_discriminant < 0) {
            return -1;
        }

        float discriminant = sqrt(square_discriminant);

        float t = (h - discriminant); //check close one first
        if (!t_bounds.surrounds(t)) {
            t = (h + discriminant); //check farther one next
            if (!t_bounds.surrounds(t)) {
                return -1;
            }
        }
        return t;
    }
}

struct Interval {
    float min;
    float max;

    float size() {
        return (this.max - this.min);
    };

    bool contains(float x) {
        return (this.min <= x && x <= this.max);
    };

    bool surrounds(float x) {
        return (this.min < x && x < this.max);
    };

    float clamp(float x) {
        return clamp(x,this.min,this.max);
    }
}

struct Random {
    uint state;

    [mutating] float random() {
        this.state = (this.state * uint(747796405)) + uint(2891336453);
        uint result = ((this.state >> ((this.state >> 28) + 4)) ^ this.state) * 277803737;
        result = (result >> 22) ^ result;

        return float(result)/(4294967296.0); // 32 bit integer total possible values
    }

    [mutating]
    float3 random_unit_vector() {
        float z = 2.0 * random() - 1.0;
        float a = 2.0 * float.getPi() * random();

        float r = sqrt(max(0.0, 1.0 - z * z));

        return float3(
            r * cos(a),
            r * sin(a),
            z
        );
    }

    [mutating]
    float2 random_disk() {
        float theta = 2 * random() * float.getPi();
        float r = sqrt(random());

        return float2(r * cos(theta), r * sin(theta));
    }

    static uint hash(uint seed) {
        uint seed1 = (seed * uint(747796405)) + uint(2891336453);
        uint result = ((seed1 >> ((seed1 >> 28) + 4)) ^ seed1) * 277803737;
        result = (result >> 22) ^ result;

        return result; // 32 bit integer total possible values}
    }

}

struct Material {
    float3 color;
    float emission_strength;
    float3 emission_color;
    float smoothness;
}