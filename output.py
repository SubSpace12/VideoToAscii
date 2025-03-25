from tkinter import *
import csv
import time
import ffmpeg
import playsound as ps
probe = ffmpeg.probe("rust.mp4")
video_streams = [stream for stream in probe["streams"] if stream["codec_type"] == "video"]
window = Tk()
window.geometry("{}x{}".format(video_streams[0]["width"], video_streams[0]["height"]))
window.title("Video output")
window.configure(bg="black")
def play():
    with open('frames.csv') as csv_file:
        csv_reader = csv.reader(csv_file, delimiter=',')
       
        for row in csv_reader:
            window.after(int(1000/int((video_streams[0]["r_frame_rate"])[:-2])), var.set(row))
            window.update()
var = StringVar()
label = Label(window, textvariable = var)
label.config(font=("Consolas", 5), fg="lime", bg="black")
label.pack()
w = Button(window, command = play, text="playback")
w.pack()
window.mainloop()