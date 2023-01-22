import requests, os
import boorus

# User-Agent header for request
headers = {'User-Agent': 'Booru2Training data a0.3'}

# Get directory to save images
print('Pick a directory to save images to, it must already exist: ')
image_save_directory = input()

# Get user's choice of booru service
print('''What Booru service would you like to use?
1. Danbooru(What most anime models are already trained on)
2. Gelbooru(Tags can be diffrent so might not be as good, however has less censorship without Gold account)
3. Gelbooru old(same API as rule34 xxx as it seems to be on older version)
Pick a number 1..3: ''')
booru_choice = int(input())

# If user chooses Gelbooru, check if they want to change default URL
if booru_choice == 2:
    print('Would you like to change the Gelbooru URL from the default(https://gelbooru.com)? (Y/n):')
    url_choice = input()
    if url_choice.upper() == 'Y':
        print('Please type a URL: ')
        gelbooru_url = input()
    elif url_choice.upper() == 'N':
        gelbooru_url = 'https://gelbooru.com/'

# If user chooses old Gelbooru, check if they want to change default URL
elif(booru_choice == 3):
    print('Would you like to change the Gelbooru URL from the default(https://rule34.xxx/)? (Y/n):')
    url_choice = input()
    if url_choice.upper() == 'Y':
        print('Please type a URL: ')
        gelbooru_url_old = input()
    elif url_choice.upper() == 'N':
        gelbooru_url_old = 'https://rule34.xxx/'

# Check if images are NSFW
print('Are your images NSFW? (Y/n): ')
nsfw_choice = input()
if nsfw_choice.upper() == 'Y':
    content_type = 'nsfw'
elif nsfw_choice.upper() == 'N':
    content_type = 'sfw'

while True:
    if booru_choice == 1:
        print('Enter a image ID for Danbooru:')
        id = input()
        post = boorus.get_image_danbooru(id, headers)
    elif booru_choice == 2:
        print('Enter a image ID for Gelbooru:')
        id = input()
        post = boorus.get_image_gelbooru(id, gelbooru_url, headers)
    elif booru_choice == 3:
        print('Enter a image ID for Gelbooru:')
        id = input()
        post = boorus.get_image_gelbooru_old(id, gelbooru_url_old, headers)

    # Make request to download image
    image_response = requests.get(post['file_url'], headers=headers)

    # Check if request was successful
    if image_response.status_code == 200:
        # Save image to specified directory
        with open(f"{image_save_directory}/{post['id']}.{post['file_ext']}", 'wb') as f:
            f.write(image_response.content)
            print('Image downloaded successfully.')

            # Save image tags to text file
            with open(f"{image_save_directory}/{post['id']}.txt", 'w') as f:
                post['tag_string'] = ', '.join([i for i in post['tag_string'].split()])
                f.write(f"masterpiece, highest quality, {content_type}, {post['tag_string']}")
                print('saved tags')
    else:
        print('Failed to download image.')

