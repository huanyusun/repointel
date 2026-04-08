from workers import login


def run_job(name: str) -> bool:
    return login(name)
