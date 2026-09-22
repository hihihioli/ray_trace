struct Textures {
    [format("rgba32f")]
    RWTexture2D<float4> result;
}

struct Constants {
    ConstantBuffer<Uniforms> uniforms;
}

struct Uniforms {
    uint frame_count;
    uint accumulated_frames;
}

ParameterBlock<Textures> textures;
ParameterBlock<Constants> constants;

[shader("compute")]
[numthreads(8,8)]
void main(uint3 threadId: SV_DispatchThreadID) {
    uint scale = 1;

    var result = textures.result;
    var frame_count = constants.uniforms.frame_count;
    var accumulated_frames = constants.uniforms.accumulated_frames;

    uint2 texture_dimensions;
    result.GetDimensions(texture_dimensions.x, texture_dimensions.y);

    if (threadId.x >= texture_dimensions.x || threadId.y >= texture_dimensions.y) {
        return;
    }

    float aspect = float(texture_dimensions.x)/float(texture_dimensions.y);

    float focal_length = 1;
    float viewport_height = 1.5;
    float viewport_width = viewport_height * aspect;
    float3 camera_center = 0;

    float delta_u = viewport_width / float(texture_dimensions.x/scale);
    float delta_v = viewport_height / float(texture_dimensions.y/scale);

    Ray ray = Ray(camera_center,float3(-viewport_width/2 + (float(threadId.x/scale) + 0.5) * delta_u,
                                       viewport_height/2 - (float(threadId.y/scale) + 0.5) * delta_v,
                                       -focal_length));

    uint pixel_index = threadId.x/scale + threadId.y/scale * texture_dimensions.x/scale;
    Random random = Random(pixel_index^hash(frame_count));

    Sphere spheres[] = {
        Sphere(100000, float3(0,-100000-0.3,0),float3(0,0,1)),
        Sphere(0.5,float3(-0.25,0,-2),float3(1,0,0)),
        Sphere(0.5,float3(0.25,0,-2),float3(0,1,0)),
    };

    // offset should be between -.5 and .5 for subpixel aa
    float2 offset = float2(random.random()-0.5, random.random()-0.5);
    float3 ray_offset_direction =  float3(-viewport_width/2 + (float(threadId.x/scale) + 0.5 + offset.x) * delta_u,
                                          viewport_height/2 - (float(threadId.y/scale) + 0.5 + offset.y) * delta_v,
                                          -focal_length);
    Ray offset_ray = Ray(camera_center,normalize(ray_offset_direction));
    float4 color = offset_ray.color(spheres);

    float3 color_accumulated = color.rgb / float(accumulated_frames + 1) +
                               result.Load(threadId.xy).rgb * float(accumulated_frames) / float(accumulated_frames + 1);


    result.Store(threadId.xy,float4(color_accumulated,color.a));
}

struct Ray {
    float3 origin;
    float3 direction;

    float3 position(float t) {
        return origin + direction * t;
    };

    float4 color(Sphere spheres[]) {
        Interval t_bounds = Interval(0,500);
        HitRecord hit_record;
        bool hit_anything = false;
        [unroll]
        for (int i = 0; i < spheres.getCount(); i++) {
            if (spheres[i].hit(this,t_bounds,hit_record)) {
                hit_anything = true;
                t_bounds = Interval(t_bounds.min,hit_record.t);
            }
        }
        if (hit_anything) {
            return float4(0.5*(hit_record.normal+float3(1)),hit_record.t);
            //return hit_record.color;
        } else {
            float a = 0.5 * (normalize(this.direction).y + 1);
            float3 color = float3(0) * (1-a) + float3(1) * (a);
            return float4(color,-1);
        }
    }
}

struct Sphere {
    float radius;
    float3 center;
    float3 color;

    bool hit(Ray ray, Interval t_bounds, out HitRecord hit) {
        float3 oc = this.center - ray.origin;

        float a = dot(ray.direction,ray.direction);
        float h = dot(ray.direction,oc);
        float c = dot(oc,oc) - this.radius * this.radius;
        float square_discriminant = h*h - a*c;

        if (square_discriminant < 0) {
            return false;
        }

        float discriminant = sqrt(square_discriminant);

        float t = (h - discriminant) / a; //check close one first
        if (!t_bounds.surrounds(t)) {
            t = (h + discriminant) / a; //check farther one next
            if (!t_bounds.surrounds(t)) {
                return false;
            }
        }
        hit.t = t;
        hit.position = ray.position(t);
        hit.normal = (hit.position - this.center) / this.radius;
        hit.color = this.color;

        return true;
    }
}
struct HitRecord {
    float3 position;
    float t;
    float3 normal;
    float3 color;
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
}

uint hash(uint seed) {
    uint seed1 = (seed * uint(747796405)) + uint(2891336453);
    uint result = ((seed1 >> ((seed1 >> 28) + 4)) ^ seed1) * 277803737;
    result = (result >> 22) ^ result;

    return result; // 32 bit integer total possible values}
}