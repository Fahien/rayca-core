// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

SubpassInput image;

[shader("fragment")]
float4 main() : SV_Target {
    return image.SubpassLoad();
}
