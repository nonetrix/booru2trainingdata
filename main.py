import requests, cv2, math, os
import numpy as np

# Ask user for image save directory and store input
print("Pick a directory to save images to, it must already exist: ")
image_save_directory = input()

# TODO
# print("Pick what booru service you would like to use: ")


print("Are your images NSFW? (Y/n): ")
nsfw_choice = input()
if(nsfw_choice.upper() == "Y"):
        content_type = "nsfw"
elif(nsfw_choice.upper() == "N"):
        content_type = "sfw"


# Loop to continuously retrieve image IDs from user
while True:
    print("Enter a image ID for Danbooru:")
    id = input()
    
    # Retrieve post data from Danbooru API
    response = requests.get(
        'https://danbooru.donmai.us/posts/' + id + '.json?login=anon&api_key=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX'
    )
    
    # Check if post data was retrieved successfully
    if response.status_code == 200:
        post = response.json()
        
        # Retrieve image from file URL
        image_response = requests.get(post['file_url'])
        
        # Check if image was retrieved successfully
        if image_response.status_code == 200:
            
            # Save image to specified directory
            with open(
                image_save_directory + '/' + str(post['id']) + '.' + post['file_ext'], 'wb'
            ) as f:
                f.write(image_response.content)

                print('Image downloaded successfully.')

                # Save image tags to a text file
                with open(
                    image_save_directory + '/' + str(post['id']) + '.txt', 'w'
                ) as f:
                    f.write(
                        'masterpiece, highest quality, ' + content_type + ' ' + post['tag_string']
                    )
                    print('saved tags')
        else:
            print('Failed to download image.')
    else:
        print('Failed to retrieve post data: ' + response)
