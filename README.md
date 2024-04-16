# OpenAI API Rust Backend

This project provides a Rust backend that serves as a foundation for integrating OpenAI's APIs into frontend applications. It currently supports the following OpenAI API functionalities:

- Chat Completion: Generate chats or other text-based content using the GPT-3.5-turbo model.
- Image Generation: Generate images using the DALL-E model.

## Prerequisites

Before running the backend, make sure you have the following:

- Rust programming language installed (version 1.x.x)
- OpenAI API key

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/your-username/openai-api-rust-backend.git
   ```

2. Change into the project directory:

   ```bash
   cd openai-api-rust-backend
   ```

3. Install the dependencies:

   ```bash
   cargo build
   ```

4. Set up your OpenAI API key:

   - Create a `.env` file in the project root directory.
   - Add the following line to the `.env` file, replacing `YOUR_API_KEY` with your actual OpenAI API key:

     ```
     OPENAI_API_KEY=YOUR_API_KEY
     ```

## Usage

1. Start the backend server:

   ```bash
   cargo run
   ```

   The server will start running on `http://localhost:8080`.

2. Use an API testing tool like Insomnia or cURL to send requests to the available endpoints:

   - **Chat Completion**

     - Endpoint: `POST /generate_chat`
     - Request Body (JSON):
       ```json
       {
         "model": "gpt-3.5-turbo",
         "messages": [
           {
             "role": "system",
             "content": "You are a poetic assistant, skilled in explaining complex programming concepts with creative flair."
           },
           {
             "role": "user",
             "content": "Compose a poem that explains the concept of recursion in programming."
           }
         ]
       }
       ```
     - Response: The generated poem as plain text.

   - **Image Generation**

     - Endpoint: `POST /generate_image`
     - Request Body (JSON):
       ```json
       {
         "model": "dall-e-3",
         "prompt": "a white siamese cat",
         "size": "1024x1024",
         "quality": "standard",
         "n": 1
       }
       ```
     - Response: The URL of the generated image.

3. Integrate the backend with your frontend application by making HTTP requests to the appropriate endpoints.

## Customization

You can customize and extend the backend functionality as needed:

- Add new endpoints for other OpenAI API features by defining new routes and handler functions in the `main` function.
- Modify the request and response structures to match your specific requirements.
- Implement additional error handling and validation as necessary.

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgements

- [OpenAI API](https://beta.openai.com/docs/api-reference/introduction)
- [Rust Programming Language](https://www.rust-lang.org/)
- [Actix Web Framework](https://actix.rs/)

Feel free to customize the README file based on your specific project details and requirements.
