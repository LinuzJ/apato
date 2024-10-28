import requests
import json
import numpy as np
import pandas as pd
import re


def extract_rent_amount(price_str):
    """
    Extracts the numerical rent amount from a price string.

    Args:
        price_str (str): The price string (e.g., "1 229 € / kk").

    Returns:
        float: The extracted rent amount as a float (e.g., 1229.0).
               Returns None if extraction fails.
    """
    # Define the regex pattern
    pattern = r"([\d\s\xa0]+)€"

    # Search for the pattern in the input string
    match = re.search(pattern, price_str)

    if match:
        # Extract the matched group
        number_str = match.group(1)

        # Remove both regular spaces and non-breaking spaces
        number_str_clean = number_str.replace("\xa0", "").replace(" ", "")

        try:
            # Convert the cleaned string to float
            rent_amount = float(number_str_clean)
            return rent_amount
        except ValueError:
            print(f"Conversion error: Unable to convert '{number_str_clean}' to float.")
            return None
    else:
        print("No match found.")
        return None


def get_auth_tokens(auth_url):
    """
    Fetch authentication tokens from the authentication API.

    Args:
        auth_url (str): The URL to fetch authentication tokens.

    Returns:
        tuple: A tuple containing 'cuid' and 'token' if successful, else (None, None).
    """
    try:
        response = requests.get(auth_url)
        response.raise_for_status()  # Raise an exception for HTTP errors
        data = response.json()
        user = data.get("user", {})
        cuid = user.get("cuid")
        token = user.get("token")
        time = user.get("time")
        if cuid and token:
            return cuid, token, str(time)
        else:
            print("Authentication tokens not found in the response.")
            return None, None
    except requests.exceptions.RequestException as e:
        print(f"Error fetching authentication tokens: {e}")
        return None, None


def fetch_property_data(search_url, headers):
    """
    Fetch property data from the search API using the provided headers.

    Args:
        search_url (str): The search API URL.
        headers (dict): The headers to include in the API request.

    Returns:
        list: A list of property cards if successful, else an empty list.
    """
    try:
        response = requests.get(search_url, headers=headers)
        response.raise_for_status()
        data = response.json()
        return data.get("cards", [])
    except requests.exceptions.RequestException as e:
        print(f"Error fetching property data: {e}")
        return []


def extract_features(cards):
    """
    Extract relevant features from the property cards.

    Args:
        cards (list): A list of property card dictionaries.

    Returns:
        list: A list of dictionaries containing extracted features.
    """
    data_list = []

    for card in cards:
        card_data = card.get("data", {})
        location = card.get("location", {})

        price_str = card_data.get("price", "")
        rent_amount = extract_rent_amount(price_str)

        size_m2 = card_data.get("sizeMin", "")
        num_rooms = card_data.get("rooms", None)
        build_year = card_data.get("buildYear", None)
        floor = card_data.get("floor", None)

        zip_code = location.get("zipCode", "")
        latitude = location.get("latitude", None)
        longitude = location.get("longitude", None)
        city = location.get("city", "")
        district = location.get("district", "")

        data_list.append(
            {
                "rent_amount": rent_amount,
                "zip_code": zip_code,
                "size_m2": size_m2,
                "num_rooms": num_rooms,
                "build_year": build_year,
                "floor": floor,
                "latitude": latitude,
                "longitude": longitude,
                "city": city,
                "district": district,
            }
        )

    return data_list


def main():
    auth_url = "https://asunnot.oikotie.fi/user/get?format=json&rand=35963"
    search_url = 'https://asunnot.oikotie.fi/api/5.0/search?cardType=101&limit=3000&locations=[[1,9,"Suomi"]]'

    cuid, token, time = get_auth_tokens(auth_url)
    if not cuid or not token or not time:
        print("Cannot proceed without authentication tokens.")
        return

    headers = {
        "OTA-loaded": time,
        "OTA-cuid": cuid,
        "OTA-token": token,
    }

    cards = fetch_property_data(search_url, headers)
    if not cards:
        print("No property data retrieved.")
        return

    data_list = extract_features(cards)

    if not data_list:
        print("No data extracted from property cards.")
        return

    df = pd.DataFrame(data_list)

    numeric_columns = [
        "rent_amount",
        "size_m2",
        "num_rooms",
        "build_year",
        "floor",
        "latitude",
        "longitude",
    ]
    for col in numeric_columns:
        df[col] = pd.to_numeric(df[col], errors="coerce")

    print("DataFrame Statistics:")
    print(df.describe())

    print("\nHead:")
    print(df.head())

    # Save to csv
    df.to_csv("rental_data_finland.csv", index=False)


if __name__ == "__main__":
    main()
