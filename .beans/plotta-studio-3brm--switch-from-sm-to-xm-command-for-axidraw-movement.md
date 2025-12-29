---
# plotta-studio-3brm
title: Switch from SM to XM command for AxiDraw movement
status: completed
type: task
priority: normal
created_at: 2025-12-29T12:28:19Z
updated_at: 2025-12-29T12:31:35Z
parent: plotta-studio-axi1
---

Replace SM command with XM command in axidraw.rs move_to() method. XM is designed for mixed-axis geometries and handles the CoreXY transform internally, simplifying our code.