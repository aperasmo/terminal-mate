def test_matches_workspace_navigation(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Take me to the terminal-mate directory.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is True
    assert payload["source"] == "local"
    assert payload["intent"] == {
        "schema_version": 1,
        "action": "change_directory",
        "parameters": {"target": "terminal-mate"},
    }


def test_matches_workspace_navigation_to_home_directory(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Take me to the home directory.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "change_directory",
        "parameters": {"target": "home"},
    }


def test_matches_recursive_shell_file_search(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me all the .sh files.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "find_files",
        "parameters": {
            "root": ".",
            "pattern": "*.sh",
            "recursive": True,
        },
    }


def test_matches_recursive_text_file_search(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me all the .txt files.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "find_files",
        "parameters": {
            "root": ".",
            "pattern": "*.txt",
            "recursive": True,
        },
    }


def test_matches_view_file_lines_range_first(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me lines 20-100 of sample.txt.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "view_file_lines",
        "parameters": {
            "path": "sample.txt",
            "start_line": 20,
            "end_line": 100,
        },
    }


def test_matches_view_file_lines_path_first(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me the content of sample.txt lines 20 to 100.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "view_file_lines",
        "parameters": {
            "path": "sample.txt",
            "start_line": 20,
            "end_line": 100,
        },
    }


def test_returns_unsupported_for_unknown_request(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Deploy everything everywhere.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is False
    assert payload["intent"] is None
