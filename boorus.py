import requests

def get_image_danbooru(id, headers):
    """
    Get image details from Danbooru
    :param id: Image ID on Danbooru
    :return: Dictionary containing image details
    """
    # Make request to Danbooru API
    response = requests.get('https://danbooru.donmai.us/posts/' + id + '.json', headers=headers)

    # Check if request was successful
    if response.status_code == 200:
        post = response.json()
        return {
            'file_url': post['file_url'],
            'file_ext': post['file_ext'],
            'id': post['id'],
            'tag_string': post['tag_string'],
        }
    else:
        print('Request failed.')

def get_image_gelbooru(id, url, headers, api_key, user_id):
    """
    Get image details from Gelbooru
    :param id: Image ID on Gelbooru
    :param url: URL to instance
    :param api_key: API key for authentication
    :param user_id: User ID for authentication
    :return: Dictionary containing image details
    """
    # Make request to Gelbooru API with API key and User ID
    request_url = f"{url}/index.php?page=dapi&s=post&q=index&json=1&id={id}&api_key={api_key}&user_id={user_id}"
    response = requests.get(request_url, headers=headers)

    # Check if request was successful
    if response.status_code == 200:
        post = response.json()
        # Gelbooru API usually returns a list under 'post', check to ensure it exists
        if 'post' in post and len(post['post']) > 0:
            return {
                'file_url': post['post'][0]['file_url'],
                'file_ext': post['post'][0]['file_url'].split(".")[-1],
                'id': post['post'][0]['id'],
                'tag_string': post['post'][0]['tags'],
            }
        else:
            print('Post not found or API structure error.')
    else:
        print(f'Request failed. Status Code: {response.status_code}')

def get_image_gelbooru_old(id, url, headers):
    """
    Get image details from Gelbooru using older API
    :param id: Image ID on Gelbooru
    :param url: URL to instance
    :return: Dictionary containing image details
    """
    # Make request to Gelbooru API
    response = requests.get(url + '/index.php?page=dapi&s=post&q=index&json=1&id=' + id, headers=headers)

    # Check if request was successful
    if response.status_code == 200:
        post = response.json()
        return {
            'file_url': post[0]['file_url'],
            'file_ext': post[0]['file_url'].split(".")[-1],
            'id': post[0]['id'],
            'tag_string': post[0]['tags'],
        }
    else:
        print('Request failed.')
