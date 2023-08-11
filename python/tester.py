from machine import Pwm, Pin

servo = Pwm(Pin(16))
servo.freq(50)
servo.duty_u16(0)