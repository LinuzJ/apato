import requests

x = requests.get('https://asunnot.oikotie.fi/user/get?format=json&rand=7123')
print(x.json())