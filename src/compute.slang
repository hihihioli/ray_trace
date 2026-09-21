struct Textures {
    [format("rgba16f")]
    WTexture2D<float4> result;
}

struct Constants {
    ConstantBuffer<Uniforms> uniforms;
}

struct Uniforms {
    uint frame_count;
}

ParameterBlock<Textures> textures;
ParameterBlock<Constants> constants;

[shader("compute")]
[numthreads(8,8)]
void main(uint3 threadId: SV_DispatchThreadID) {
    var result = textures.result;
    uint2 texture_dimensions;
    result.GetDimensions(texture_dimensions.x, texture_dimensions.y);

    if (threadId.x >= texture_dimensions.x || threadId.y >= texture_dimensions.y) {
        return;
    }

    float2 uv = float2(float(threadId.x) / texture_dimensions.x, float(threadId.y) / texture_dimensions.y);
    float aspect = float(texture_dimensions.x)/float(texture_dimensions.y);

    float focal_length = 1;
    float viewport_height = 2;
    float viewport_width = viewport_height * aspect;
    float3 camera_center = 0;

    float delta_u = viewport_width / float(texture_dimensions.x);
    float delta_v = viewport_height / float(texture_dimensions.y);

    Ray ray = Ray(camera_center,float3(-viewport_width/2 + (float(threadId.x) + 0.5) * delta_u,
                                       viewport_height/2 - (float(threadId.y) + 0.5) * delta_v,
                                       -focal_length));

    //float3 color = ray.color();
    uint pixel_index = threadId.x + threadId.y * texture_dimensions.x;
    Random random = Random(pixel_index^hash(constants.uniforms.frame_count));
    float3 color = float3(random.random(),random.random(),random.random());


    result.Store(threadId.xy,float4(color,1));
}

struct Ray {
    float3 origin;
    float3 direction;

    float3 position(float t) {
        return origin + direction * t;
    };

    float3 color(int samples) {
        Sphere spheres[] = {
            Sphere(100000, float3(0,-100000-0.3,0),float3(0,0,1)),
            Sphere(0.5,float3(-0.25,0,-1),float3(1,0,0)),
            Sphere(0.5,float3(0.25,0,-1),float3(0,1,0)),
        };
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
            return 0.5*(hit_record.normal+float3(1));
            //return hit_record.color;
        } else {
            float a = 0.5 * (normalize(this.direction).y + 1);
            return float3(0) * (1-a) + float3(1) * (a);
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