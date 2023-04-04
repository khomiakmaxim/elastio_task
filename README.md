# ELASTIO_TASK
CLI application for weather data requests. Configurable, scalable, and simple. Developed with Rust as a test project.

## How to run
## 1. Clone current repository
``
git clone git@github.com:khomiakmaxim/elastio_task.git
``
## 2. Add a '.env' file in the elastio_task repository, with the following content:
### OPEN_WEATHER_MAP=your_open_weather_api_key
### WEATHER_API=your_weather_api_key
For receiving 'your_open_weather_api_key', make sure to register https://openweathermap.org/api/one-call-3 api_key, since the application works with only this type of API key from the current provider. For receiving 'your_weather_api_key' register simplest possible api_key from https://www.weatherapi.com. For now, this is the only deviation from https://gist.github.com/anelson/0029f620105a19702b5eed5935880a28 task.
## 3. Build project
``
cargo build
``
## 4. Run tests
``
cargo test
``

``
cargo test -- --ignored
``

All tests which marked ignored are API call tests.
## 5. Run application and follow instructions from the help command. Input commands expected to be after 'cargo run --'
``
cargo run -- help
``

## You might find the documentation in 
``/target/doc/elastio_task/``
