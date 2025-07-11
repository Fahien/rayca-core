// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

[vk::input_attachment_index(0)]
SubpassInput color;
[vk::input_attachment_index(1)]
SubpassInput normal;
[vk::input_attachment_index(2)]
SubpassInput depth;

[shader("fragment")]
float4 main() : SV_Target {
    float4 d = depth.SubpassLoad();
    return float4(d.x, d.x, d.x, 1.0);
}
