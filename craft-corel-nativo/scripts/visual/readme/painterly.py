# The Painterly Brush tool with a grass stroke picked and the brush
# picker open.
time.sleep(0.5)
move(15, 272)
xtest.fake_input(d, X.ButtonPress, 1); d.sync(); time.sleep(1.2)
xtest.fake_input(d, X.ButtonRelease, 1); d.sync(); time.sleep(0.6)
click(100, 299); time.sleep(0.6)
click(640, 640); time.sleep(0.8)
click(90, 88); time.sleep(0.8)
move(1300, 900); time.sleep(0.5)
shot("painterly")
