CREATE DATABASE colours_test;

CREATE USER colourtester WITH PASSWORD 'password';
    
GRANT ALL ON DATABASE colours_test TO colourtester;