FROM python:3.12-slim

WORKDIR /srv
COPY server.py /srv/server.py

# Both ports are handled by the same process; expose the pair the crawler uses.
EXPOSE 8000 8001 8002

CMD ["python", "-u", "/srv/server.py"]
