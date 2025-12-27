---
# plotta-studio-axi3
title: Implement EBB commands
status: completed
type: task
priority: normal
created_at: 2025-12-23T19:00:00Z
updated_at: 2025-12-27T13:42:10Z
parent: plotta-studio-axi1
---

Implement the core EBB protocol commands for pen control and movement.

## EBB Commands to Implement

### Pen Control (SP command)
```rust
/// SP,value,duration,portB
/// value: servo position (typically 0-255, but 25-75 for pen)
/// duration: transition time in ms
pub fn pen_up(&mut self) -> Result<(), PlotterError> {
    let cmd = format!("SP,0,{},{}", self.config.pen_up_pos, self.config.pen_up_delay);
    self.send_command(&cmd)?;
    std::thread::sleep(Duration::from_millis(self.config.pen_up_delay as u64));
    self.pen_is_down = false;
    Ok(())
}

pub fn pen_down(&mut self) -> Result<(), PlotterError> {
    let cmd = format!("SP,1,{},{}", self.config.pen_down_pos, self.config.pen_down_delay);
    self.send_command(&cmd)?;
    std::thread::sleep(Duration::from_millis(self.config.pen_down_delay as u64));
    self.pen_is_down = true;
    Ok(())
}
```

### Stepper Movement (SM command)
```rust
/// SM,duration,axis1_steps,axis2_steps
/// AxiDraw uses CoreXY-style: axis1=A+B, axis2=A-B
const STEPS_PER_MM: f64 = 80.0; // 16 microsteps * 200 steps/rev / 40mm per rev

pub fn move_to(&mut self, target: Point) -> Result<(), PlotterError> {
    let delta = target - self.current_pos;
    let distance = delta.length();

    if distance < 0.01 {
        return Ok(());
    }

    // Calculate steps
    let steps_x = (delta.x * STEPS_PER_MM) as i32;
    let steps_y = (delta.y * STEPS_PER_MM) as i32;

    // CoreXY transform
    let axis1 = steps_x + steps_y;
    let axis2 = steps_x - steps_y;

    // Calculate duration based on speed
    let speed = if self.pen_is_down {
        self.config.pen_down_speed
    } else {
        self.config.pen_up_speed
    };
    let duration_ms = ((distance / speed) * 1000.0) as u32;

    let cmd = format!("SM,{},{},{}", duration_ms, axis1, axis2);
    self.send_command(&cmd)?;
    std::thread::sleep(Duration::from_millis(duration_ms as u64));

    self.current_pos = target;
    Ok(())
}
```

### Motor Control (EM command)
```rust
pub fn disable_motors(&mut self) -> Result<(), PlotterError> {
    self.send_command("EM,0,0")?;
    Ok(())
}

pub fn enable_motors(&mut self) -> Result<(), PlotterError> {
    self.send_command("EM,1,1")?;
    Ok(())
}
```

## Files to Modify
- `crates/drawing-plotter/src/lib.rs`
