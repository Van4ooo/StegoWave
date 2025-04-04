# StegoWave
Audio steganography services with a cli client written in Rust🦀

### Available audio file formats

| Format                 | CLI code  | Version | Implemented                                                                                                 |
|------------------------|-----------|---------|-------------------------------------------------------------------------------------------------------------|
| WAV (16-bits samples)  | **wav16** | ^0.0.1  | [WAV16](https://github.com/Van4ooo/StegoWave/blob/capstone-project/stego_wave/src/formats/wav.rs#L117-L430) |

### Available services

| Architecture | CLI code | Directory                                                                                | API                                                                                                              |
|--------------|----------|------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------|
| REST         | **rest** | [/rest_service](https://github.com/Van4ooo/StegoWave/tree/capstone-project/rest_service) | [Swagger](http://localhost:8080/swagger-ui/)                                                                     |
| gRPC         | **grpc** | [/grpc_service](https://github.com/Van4ooo/StegoWave/tree/capstone-project/grpc_service) | [Proto file](https://github.com/Van4ooo/StegoWave/blob/capstone-project/grpc_service/proto/stego_wave.proto#L35) |

---

### Steganography core lib => [stego_wave](https://github.com/Van4ooo/StegoWave/tree/capstone-project/stego_wave)

This Rust library provides audio steganography functionality. It allows users to hide, extract, and clear secret messages within audio samples using least significant bit (LSB) manipulation.
Below is an overview of its key features and design:

#### Key Features
- **WAV16 Format Support:**
The library currently supports only 16-bit WAV audio files. It validates that the file meets the 16-bit requirement before processing.

- **Message Embedding:**
The library can embed secret messages into audio files by altering the LSBs of audio samples. A header (e.g., "STEG") is embedded along with the message to ensure correct extraction.

    - Methods include ```hide_message``` for processing files and ```hide_message_binary``` for direct sample manipulation.

- **Message Extraction:**
It provides methods to retrieve the hidden message from an audio file.

  - The methods ```extract_message``` and ```extract_message_binary``` work by reading the LSBs and reconstructing the hidden data.
    
- **Message Clearing:**
In addition to embedding and extraction, the library can clear the hidden message from an audio file using the provided password to locate the modified samples.

- **Password-Based Indexing:**
Unique random indices are generated based on a password, ensuring that only users with the correct password can extract or clear the hidden message.

- **Configurable LSB Depth:**
  Users can configure the number of LSBs used (with a default of 1 and a valid range of 1 to 16) to balance between audio quality and message capacity. This is implemented using the builder pattern for easy and flexible configuration.

---

## Build setup

Getting the git repository
```shell
git clone --branch capstone-project --single-branch https://github.com/Van4ooo/StegoWave.git
cd StegoWave
```

Run gRPC server
```shell
cargo run --bin grpc_server
```

Run REST server
```shell
# Fix zip v2.5 error. https://github.com/juhaku/utoipa/issues/1341
cargo update zip --precise 2.2.2
cargo run --bin rest_server
```
You can test the server's REST API at [Swagger](http://localhost:8080/swagger-ui/)

---

## Tutorial 

```sw``` - is a Rust-based tool that allows you to hide, extract, and clear secret message within audio file. It leverages different servers (gRPC and REST) to process your requests and uses least significant bit (LSB) steganography techniques to embed or remove information.

### Prerequisites

- **Rust Toolchain:** Make sure you have Rust installed. You can download it from [site](https://www.rust-lang.org/tools/install).

- **Audio File**: Have an audio file (e.g., WAV format) ready for testing.

- **Server Setup:** The application connects to a server running locally. By default:

  - **gRPC** server URL: ```http://[::1]:50051```

  - **REST** server URL: ```http://127.0.0.1:8080```

  Ensure that at least one of these servers is running or use the ```auto``` option to let the application try gRPC first and then REST if needed.

#### Build ```sw```
```shell
cargo build --release --bin sw
./target/release/sw --help
```

![image](https://github.com/user-attachments/assets/7b1a63d2-bde8-4e6c-bac9-bde9ab9cbd52)



### Usage ```sw```
```sw``` supports three primary commands: ```hide```, ```extract```, and ```clear```. All commands share some common fields such as the input file name,
audio file format, server choice, and the number of LSBs to modify.

### Hiding a secret message
Use the ```hide``` command to embed a secret message into an audio file.

```shell
./target/release/sw hide --help
```

![image](https://github.com/user-attachments/assets/3b23d673-98cd-4801-bed3-1a1cda881764)


- #### Windows command syntax
    ```shell
    ./target/release/sw hide --input_file <FILE_NAME> --output_file <FILE_NAME> --message "Super secret message!!" --format wav16
    ``` 
- #### Linux command syntax
    ```shell
    cat <FILE_NAME> | sw hide --message "Super secret message!!" --format wav16 > <FILE_NAME>
    ```


### Extracting a secret message
Use the ```extract``` command to retrieve a hidden message from an audio file.

```shell
./target/release/sw extract --help
```

![image](https://github.com/user-attachments/assets/f7b51cd4-7fe4-4fb1-9906-a28c83b9ffdd)


- #### Windows command syntax
    ```shell
    sw extract --input_file <FILE_NAME> --format wav16
    ``` 
- #### Linux command syntax
    ```shell
    cat <FILE_NAME> | sw extract  --format wav16 > rez.txt
    ```


### Clearing a hidden message
The ```clear``` command removes the hidden message from an audio file.

```shell
./target/release/sw clear --help 
```

![image](https://github.com/user-attachments/assets/6d135f83-5499-4f8a-9c3b-ae13cf980886)

- #### Windows command syntax
    ```shell
    sw clear --input_file <FILE_NAME> --output_file <FILE_NAME> --format wav16
    ``` 
- #### Linux command syntax
    ```shell
    cat <FILE_NAME> | sw clear --format wav16 > <FILE_NAME>
    ```

### Automatically start the server if it's not running
```shell
sw <COMMAND> --start-server --server grpc
sw <COMMAND> --start-server --server rest
```
--- 
This tutorial should help you understand and use the ```sw``` application for embedding, extracting,
and clearing hidden messages within audio files. Happy steganography!
