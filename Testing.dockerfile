FROM navrin/colour-bot-v2-base:latest
USER root

CMD ["bash", "-c", "chmod +x /app/docker-test.sh && /app/docker-test.sh"]