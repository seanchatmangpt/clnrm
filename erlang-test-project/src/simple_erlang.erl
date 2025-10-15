-module(simple_erlang).
-export([hello/0, add/2, json_response/0, server_status/0]).

%% Simple hello function
hello() ->
    "Hello from Erlang!".

%% Simple arithmetic function
add(A, B) ->
    A + B.

%% JSON-like response function
json_response() ->
    "{\"message\":\"Hello from Erlang\",\"status\":\"running\",\"version\":\"1.0.0\"}".

%% Server status function
server_status() ->
    "{\"service\":\"erlang-test\",\"status\":\"healthy\",\"uptime\":\"0\"}".

%% Simple HTTP server function (basic implementation)
start_server(Port) ->
    io:format("Starting simple Erlang server on port ~p~n", [Port]),
    case gen_tcp:listen(Port, [binary, {packet, http}, {active, false}, {reuseaddr, true}]) of
        {ok, ListenSocket} ->
            io:format("Server listening on port ~p~n", [Port]),
            accept_loop(ListenSocket);
        {error, Reason} ->
            io:format("Failed to start server: ~p~n", [Reason])
    end.

accept_loop(ListenSocket) ->
    case gen_tcp:accept(ListenSocket) of
        {ok, Socket} ->
            spawn(fun() -> handle_connection(Socket) end),
            accept_loop(ListenSocket);
        {error, closed} ->
            ok;
        {error, Reason} ->
            io:format("Accept error: ~p~n", [Reason])
    end.

handle_connection(Socket) ->
    case gen_tcp:recv(Socket, 0) of
        {ok, Request} ->
            Response = generate_response(Request),
            gen_tcp:send(Socket, Response),
            gen_tcp:close(Socket);
        {error, Reason} ->
            io:format("Receive error: ~p~n", [Reason])
    end.

generate_response(Request) ->
    case parse_request(Request) of
        {get, "/"} ->
            build_response(200, "text/plain", "Erlang Test Server - Hello World!");
        {get, "/hello"} ->
            build_response(200, "text/plain", hello());
        {get, "/add/2/3"} ->
            Result = add(2, 3),
            build_response(200, "text/plain", integer_to_list(Result));
        {get, "/json"} ->
            build_response(200, "application/json", json_response());
        {get, "/health"} ->
            build_response(200, "application/json", server_status());
        _ ->
            build_response(404, "text/plain", "Not Found")
    end.

parse_request(Request) ->
    Lines = binary:split(Request, <<"\r\n">>, [global]),
    case Lines of
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

build_response(StatusCode, ContentType, Body) ->
    StatusText = case StatusCode of
        200 -> "OK";
        404 -> "Not Found";
        _ -> "Unknown"
    end,

    Response = io_lib:format(
        "HTTP/1.1 ~p ~s\r\nContent-Type: ~s\r\nContent-Length: ~p\r\n\r\n~s",
        [StatusCode, StatusText, ContentType, length(Body), Body]
    ),

    list_to_binary(Response).
