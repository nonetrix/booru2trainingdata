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

def get_image_gelbooru(id, url, headers):
    """
    Get image details from Gelbooru
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
            'file_url': post['post'][0]['file_url'],
            'file_ext': post['post'][0]['file_url'].split(".")[-1],
            'id': post['post'][0]['id'],
            'tag_string': post['post'][0]['tags'],
        }
    else:
        print('Request failed.')

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
