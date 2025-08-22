#!/bin/bash

# Source the .env file if it exists
if [ -f .env ]; then
    source .env
fi

if [ -z "$REPLICATE_API_TOKEN" ]; then
    echo "Error: REPLICATE_API_TOKEN not set"
    echo "Please create a .env file with your token:"
    echo "REPLICATE_API_TOKEN=your_token_here"
    exit 1
fi

# Create directory for scene images
mkdir -p gnome-extension/elgato-ring-light@example.com/images

# Scene definitions with prompts
declare -A scenes=(
    ["daylight"]="bright sunny office workspace with natural daylight streaming through windows, professional photography, warm and productive atmosphere, 5600K color temperature lighting"
    ["warm"]="cozy evening living room with warm orange sunset glow, soft lighting, comfortable and relaxing, golden hour photography, 3200K warm lights"
    ["cool"]="modern minimalist workspace with cool blue-white LED lighting, crisp and clean, high-tech office environment, 6500K color temperature"
    ["reading"]="comfortable reading nook with perfect task lighting, book on desk, focused beam of light, library atmosphere, 4500K neutral light"
    ["video"]="professional video production studio setup with ring light, camera equipment, content creator workspace, balanced 5000K lighting"
)

echo "Generating scene images..."

for scene in "${!scenes[@]}"; do
    prompt="${scenes[$scene]}"
    output_file="gnome-extension/elgato-ring-light@example.com/images/${scene}.jpg"
    
    echo "Generating image for scene: $scene"
    echo "Prompt: $prompt"
    
    # Make API call to Replicate
    response=$(curl --silent --show-error https://api.replicate.com/v1/models/black-forest-labs/flux-schnell/predictions \
        --request POST \
        --header "Authorization: Bearer $REPLICATE_API_TOKEN" \
        --header "Content-Type: application/json" \
        --header "Prefer: wait" \
        --data @- <<EOM
{
    "input": {
        "prompt": "$prompt"
    }
}
EOM
    )
    
    # Extract image URL from response
    image_url=$(echo "$response" | grep -o '"output":\["[^"]*"' | sed 's/"output":\["//')
    
    if [ -z "$image_url" ]; then
        # Try alternative parsing for stream response
        image_url=$(echo "$response" | grep -o '"stream":"[^"]*"' | sed 's/"stream":"//')
    fi
    
    if [ -n "$image_url" ]; then
        echo "Downloading image from: $image_url"
        curl -s -o "$output_file" "$image_url"
        echo "Saved to: $output_file"
    else
        echo "Failed to get image URL for $scene"
        echo "Response: $response"
    fi
    
    echo "---"
    sleep 1
done

echo "All scene images generated!"