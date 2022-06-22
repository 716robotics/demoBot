# demoBot
Non-competition spec demo bot (Gen 1) powered by raspberry pi, written in rust

## operation
Drivetrain is controlled by the left stick in arcade drive. Shoot by holding the A button (full power) or the B button (60% power). E-stop by pressing B and lift the E-Stop by pressing Y.

## saftey
Saftey features include a watchdog in the PWM thread that disables the drivetrain after 2 seconds of no communication from the main thread and an automatic emergency stop if the controller disconnects. Note that it can take around 3.5 seconds for gilrs to detect that the controller has disconnected, so for the 2 seconds before watchdog activation, robot will be uncontrollable if a disconnect happens.
