.PHONY: help build run stop clean logs dev

help:
	@echo "📦 Доступные команды:"
	@echo "  make build    - Собрать Docker образ"
	@echo "  make run      - Запустить контейнер"
	@echo "  make stop     - Остановить контейнер"
	@echo "  make clean    - Удалить контейнер и образ"
	@echo "  make logs     - Показать логи"
	@echo "  make dev      - Запустить в режиме разработки (локально)"

build:
	docker build -t diploma-verify-frontend .

run:
	docker run -d -p 3000:80 --name diploma-verify-frontend diploma-verify-frontend

stop:
	docker stop diploma-verify-frontend || true

clean: stop
	docker rm diploma-verify-frontend || true
	docker rmi diploma-verify-frontend || true

logs:
	docker logs -f diploma-verify-frontend

dev:
	npm run dev

compose-up:
	docker-compose up --build

compose-down:
	docker-compose down