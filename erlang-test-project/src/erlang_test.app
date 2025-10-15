{application, erlang_test,
 [{description, "Erlang Test Application for Cleanroom Testing"},
  {vsn, "1.0.0"},
  {modules, [erlang_test_app]},
  {applications, [kernel, stdlib]},
  {mod, {erlang_test_app, []}}]}.
