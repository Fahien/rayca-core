// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

[shader("vertex")]
float4 main(float2 pos) : SV_Position
{
    return float4(pos.x, pos.y, 0.0, 1.0);
}
