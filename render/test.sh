# curl -s -H "Content-Type: application/geo+json"  https://api.weather.gov/points/40.722,-74.038
# curl -s -H "Content-Type: application/geo+json"  https://api.weather.gov/gridpoints/OKX/32,35/stations
# curl -s -H "Content-Type: application/geo+json"  https://api.weather.gov/zones/forecast/NJZ006/observations

# KEWR
# curl -s -H "Content-Type: application/geo+json"  https://api.weather.gov/gridpoints/OKX/32,35/forecast
# curl -s -H "Content-Type: application/geo+json"  https://api.weather.gov/gridpoints/OKX/32,35/forecast/hourly
curl -s -H "Content-Type: application/geo+json"  https://api.weather.gov/stations/KEWR/observations?limit=1
