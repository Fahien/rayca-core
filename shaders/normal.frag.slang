// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

[vk::input_attachment_index(0)]
SubpassInput color;
[vk::input_attachment_index(1)]
SubpassInput normal;

[shader("fragment")]
float4 main() : SV_Target {
    return normal.SubpassLoad();
}
