def test_health_requires_authentication(client):
    response = client.get("/v1/health")
    assert response.status_code == 401


def test_health_returns_version(client, auth_headers):
    response = client.get("/v1/health", headers=auth_headers)
    assert response.status_code == 200
    assert response.json() == {
        "status": "ok",
        "protocol_version": "1",
        "service_version": "0.1.0",
    }

