build: 
	docker-compose build

upload:
	docker-compose push

test: 
	docker-compose -f ./docker-compose.test.yml run test

coverage:
	docker-compose -f ./docker-compose.test.yml run tarpaulin cargo tarpaulin --ciserver travis-ci --coveralls $(TRAVIS_JOB_ID)