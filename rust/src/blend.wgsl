struct BlendInfo {
  x_num:u32,
  y_num:u32,
  img_width:u32,
  img_height:u32,
  imagery_layer_num:u32,
  tile_num:u32,
  tile_size:u32,
};
@group(0) @binding(0) var from_tex: texture_2d<u32>;
@group(0) @binding(1) var to_tex: texture_storage_2d<rgba8uint, write>;
@group(0) @binding(2) var<uniform> blend_info: BlendInfo;

fn blend(me:vec4u,other:vec4u)->vec4u{
  if (other.a==0){
    return me;
  }
  if (other.a==255){
    return other;
  }
  let max_t = f32(255);
  var me_f32 = vec4f(f32(me.r),f32(me.g),f32(me.b),f32(me.a));
  var other_f32 = vec4f(f32(other.r),f32(other.g),f32(other.b),f32(other.a));
  me_f32 = me_f32 / max_t;
  other_f32 = other_f32 / max_t;

  let alpha_final = me_f32.a + other_f32.a - me_f32.a * other_f32.a;
  if (alpha_final==0.0){
    return me;
  }

  let me_rgb = me_f32.rgb * me_f32.a;
  let other_rgb = other_f32.rgb * other_f32.a;

  var out_rgb = other_rgb + me_rgb * (1.0 - other_f32.a);
  out_rgb = out_rgb / alpha_final;
  var out_u32 = vec4u(
    u32(out_rgb.r * max_t),
    u32(out_rgb.g * max_t),
    u32(out_rgb.b * max_t),
    u32(alpha_final * max_t),
  );
  return out_u32;
}
@compute @workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let uv = vec2<u32>(global_id.xy);
    if (uv.x >= blend_info.tile_size || uv.y >= blend_info.tile_size) {
      return;
    }
    for(var tile_index = 0u; tile_index < blend_info.tile_num; tile_index++){
      let x_offset = tile_index * blend_info.tile_size;
      var color = vec4u(0);
      for(var layer_index = 0u; layer_index < blend_info.imagery_layer_num; layer_index++){
        let y_offset = layer_index * blend_info.tile_size;
        let origin = vec2u(x_offset,y_offset);
        let new_uv = origin+uv;
        var source_color = textureLoad(from_tex, new_uv,0);
        color = blend(color,source_color);
      }
      let y = tile_index % blend_info.y_num;
      let x = (tile_index - y) / blend_info.y_num;//TODO 3
      let uv_store = vec2u(
        uv.x + x * blend_info.tile_size,
        uv.y + y * blend_info.tile_size
      );
      textureStore(to_tex, uv_store, color);
    }
}