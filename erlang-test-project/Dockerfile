# Simple Erlang Test Application Dockerfile
FROM erlang:25-alpine

# Install necessary packages
RUN apk add --no-cache curl wget

# Create application directory
WORKDIR /app

# Copy Erlang source files
COPY src/ ./src/

# Compile the Erlang code
RUN erlc src/*.erl

# Expose port for the web server
EXPOSE 8080

# Command to run the application
CMD ["erl", "-eval", "simple_erlang:start_server(8080)", "-noshell"]
