import requests, os, sys
import boorus

# User-Agent header for request
headers = {'User-Agent': 'Booru2Training data a0.3'}

# --- Directory Handling ---
print('Pick a directory to save images to: ')
image_save_directory = input()

if not os.path.exists(image_save_directory):
    print(f"The directory '{image_save_directory}' does not exist. Would you like to create it? (Y/n): ")
    create_dir = input()
    if create_dir.upper() == 'Y':
        try:
            os.makedirs(image_save_directory)
            print(f"Created directory: {image_save_directory}")
        except OSError as e:
            print(f"Error creating directory: {e}")
            sys.exit(1)
    else:
        print("Directory required. Exiting.")
        sys.exit(1)
# --------------------------

# Get user's choice of booru service
print('''What Booru service would you like to use?
1. Danbooru(What most anime models are already trained on)
2. Gelbooru(Tags can be diffrent so might not be as good, however has less censorship without Gold account)
3. Gelbooru old(same API as rule34 xxx as it seems to be on older version)
Pick a number 1..3: ''')
booru_choice = int(input())

# Initialize variables for Gelbooru auth
gb_api_key = ''
gb_user_id = ''

# If user chooses Gelbooru, check if they want to change default URL and ask for Auth
if booru_choice == 2:
    print('Would you like to change the Gelbooru URL from the default(https://gelbooru.com)? (Y/n):')
    url_choice = input()
    if url_choice.upper() == 'Y':
        print('Please type a URL: ')
        gelbooru_url = input()
    elif url_choice.upper() == 'N':
        gelbooru_url = 'https://gelbooru.com/'
    
    # Ask for API credentials for Gelbooru
    print('Please enter your Gelbooru API Key:')
    gb_api_key = input()
    print('Please enter your Gelbooru User ID:')
    gb_user_id = input()

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
    post = None  # Reset post variable
    
    if booru_choice == 1:
        print('Enter a image ID for Danbooru:')
        id = input()
        post = boorus.get_image_danbooru(id, headers)
    elif booru_choice == 2:
        print('Enter a image ID for Gelbooru:')
        id = input()
        post = boorus.get_image_gelbooru(id, gelbooru_url, headers, gb_api_key, gb_user_id)
    elif booru_choice == 3:
        print('Enter a image ID for Gelbooru:')
        id = input()
        post = boorus.get_image_gelbooru_old(id, gelbooru_url_old, headers)

    if post:
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
                    # 1. Convert space-separated API tags to comma-separated
                    processed_tags = ', '.join([i for i in post['tag_string'].split()])
                    
                    # 2. Replace underscores with spaces
                    processed_tags = processed_tags.replace('_', ' ')
                    
                    # 3. Escape parentheses (Fixed SyntaxWarning by using double backslash)
                    processed_tags = processed_tags.replace('(', '\\(').replace(')', '\\)')
                    
                    f.write(f"masterpiece, highest quality, {content_type}, {processed_tags}")
                    print('saved tags')
        else:
            print('Failed to download image.')
    else:
        print('Failed to retrieve post data.')
