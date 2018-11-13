build: 
	docker-compose build

upload:
	docker-compose push

test: 
	docker-compose -f ./docker-compose.test.yml run test

coverage:
	docker-compose -f ./docker-compose.test.yml run tarpaulin cargo tarpaulin --ciserver travis-ci --coveralls $(TRAVIS_JOB_ID)


DEP_LIST = libcairo2-dev\
    libgtk-3-dev\
    libpango-1.0-0\
    libpangocairo-1.0-0\
    postgresql-client\
	pkg-config \
	libssl-dev \
	openssl

get_deps:
	apt-get update &&\
    apt-get install -y $(DEP_LIST)