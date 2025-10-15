-module(erlang_test_app).
-export([start/1]).

start(Port) ->
    io:format("Starting Erlang Test Application on port ~p~n", [Port]),

    % Start the HTTP server
    case start_http_server(Port) of
        {ok, Pid} ->
            io:format("HTTP server started successfully on port ~p~n", [Port]),
            % Keep the process alive
            receive
                _ -> ok
            end;
        {error, Reason} ->
            io:format("Failed to start HTTP server: ~p~n", [Reason]),
            {error, Reason}
    end.

start_http_server(Port) ->
    % For simplicity, we'll use a basic HTTP server implementation
    % In a real application, you might use Cowboy or other web frameworks
    spawn(fun() -> http_server_loop(Port) end).

http_server_loop(Port) ->
    {ok, ListenSocket} = gen_tcp:listen(Port, [binary, {packet, http}, {active, false}, {reuseaddr, true}]),
    io:format("HTTP server listening on port ~p~n", [Port]),
    accept_connections(ListenSocket).

accept_connections(ListenSocket) ->
    case gen_tcp:accept(ListenSocket) of
        {ok, Socket} ->
            spawn(fun() -> handle_request(Socket) end),
            accept_connections(ListenSocket);
        {error, Reason} ->
            io:format("Accept failed: ~p~n", [Reason])
    end.

handle_request(Socket) ->
    case gen_tcp:recv(Socket, 0) of
        {ok, Request} ->
            Response = handle_http_request(Request),
            gen_tcp:send(Socket, Response),
            gen_tcp:close(Socket);
        {error, Reason} ->
            io:format("Receive failed: ~p~n", [Reason])
    end.

handle_http_request(Request) ->
    case parse_http_request(Request) of
        {get, "/"} ->
            build_http_response(200, "text/plain", "Erlang Test Application - Hello World!");
        {get, "/health"} ->
            build_http_response(200, "application/json", "{\"status\":\"healthy\",\"service\":\"erlang-test\"}");
        {get, "/api/test"} ->
            build_http_response(200, "application/json", "{\"test\":\"passed\",\"result\":\"success\"}");
        {post, "/api/data"} ->
            build_http_response(201, "application/json", "{\"received\":true,\"status\":\"created\"}");
        {get, "/api/users"} ->
            build_http_response(200, "application/json", "[{\"id\":1,\"name\":\"Test User\"},{\"id\":2,\"name\":\"Another User\"}]");
        _ ->
            build_http_response(404, "text/plain", "Not Found")
    end.

parse_http_request(Request) ->
    case binary:split(Request, <<"\r\n">>) of
        [FirstLine | _] ->
            case binary:split(FirstLine, <<" ">>) of
                [Method, Path, _] ->
                    {binary_to_atom(Method, latin1), binary_to_list(Path)};
                _ ->
                    unknown
            end;
        _ ->
            unknown
    end.

build_http_response(StatusCode, ContentType, Body) ->
    StatusText = case StatusCode of
        200 -> "OK";
        201 -> "Created";
        404 -> "Not Found";
        _ -> "Unknown"
    end,

    Response = io_lib:format(
        "HTTP/1.1 ~p ~s\r\nContent-Type: ~s\r\nContent-Length: ~p\r\n\r\n~s",
        [StatusCode, StatusText, ContentType, length(Body), Body]
    ),

    list_to_binary(Response).
