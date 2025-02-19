import cv2
import numpy as np

# Load the image
img1 = cv2.imread('hello_0.png')
img2 = cv2.imread('hello_1.png')


# Combine the image

# numpy horizontal stack
combine_img = np.concatenate((img1, img2), axis=1)

# save the image
cv2.imwrite('combine_img.jpg', combine_img)