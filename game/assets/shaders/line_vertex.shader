#version 430 core

layout(std430, binding = 0) buffer TVertex
{
    vec4 vertex[];
};

uniform vec2 offset;
uniform vec4 surface_color;
uniform mat4 TRS;
uniform mat4 VIEW;
uniform mat4 PROJ;
uniform float thickness;

out vec4 color;
out vec3 world_position;
out vec2 uv;

const vec2 uv_table[6] = vec2[] (
vec2(0, 0), // ti = 0
vec2(0, 1), // ti = 1
vec2(1, 0), // ti = 2
vec2(1, 0), // ti = 3
vec2(0, 1), // ti = 4
vec2(1, 1)// ti = 5
);

void main()
{
    int i = gl_VertexID;
    vec4 pos = vec4(0, 0, 0, 1);

    // polyline index
    int pi = i / 6;
    // triangle index [0;5]
    int ti = i % 6;

    // direction of the extruded vertex
    float dir = ti == 0 || ti == 2 || ti == 3 ? -1.0 : 1.0;
    vec4 segment = vertex[pi+1] - vertex[pi];

    vec3 normal = normalize(vec3(-segment.y, segment.x, 0));
    vec3 mitter_normal;
    vec3 extruded_point;

    if (ti == 0 || ti == 1 || ti == 4) {
        extruded_point = vertex[pi].xyz;
        vec3 prev = pi - 1 >= 0 ? (vertex[pi] - vertex[pi - 1]).xyz : segment.xyz;
        vec3 prev_norm = normalize(vec3(-prev.y, prev.x, 0));
        mitter_normal = normalize(prev_norm + normal);
    } else {
        extruded_point = vertex[pi + 1].xyz;
        vec3 next = pi + 2 < vertex.length() ? (vertex[pi + 2] - vertex[pi + 1]).xyz : segment.xyz;
        vec3 next_n = normalize(vec3(-next.y, next.x, 0));
        mitter_normal = normalize(next_n + normal);
    }

    vec3 mitter_vec = extruded_point + mitter_normal * (thickness * 0.5 * dir) / dot(mitter_normal, normal);
    pos = vec4(mitter_vec.xyz, 1) - vec4(offset.xy, 0, 0);
    gl_Position = PROJ * VIEW * TRS * pos;

    world_position = (TRS * pos).xyz;
    uv = uv_table[ti];
    color = surface_color;
}