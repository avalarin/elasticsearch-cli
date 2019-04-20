# Elasticsearch CLI 
[![Build Status](https://travis-ci.org/avalarin/elasticsearch-cli.svg?branch=master)](https://travis-ci.org/avalarin/elasticsearch-cli)
[![Coverage Status](https://coveralls.io/repos/github/avalarin/elasticsearch-cli/badge.svg?branch=refactor-config)](https://coveralls.io/github/avalarin/elasticsearch-cli?branch=refactor-config)

Command-line interface for ElasticSearch

## Installing

### MacOS

TBD

### Linux

TBD

### From archive

* Download the latest release from the [releases page](https://github.com/avalarin/elasticsearch-cli/releases/latest)
* Unpack it to the executable files directory (e.g. /usr/local/bin)
* Make the elastic-cli binary executable `chmod +x /usr/local/bin/elastic-cli`

## Configuration

Configuration file stored in your home directory - `~/.elastic-cli`. Empty configuration file will be created at the first lanuch.

You need to register some elasticsearch server and set is as default:
```
elastic-cli config add server local --address http://localhost:9200 --index '*'
elastic-cli config use server local
```

## Usage

Examples:
```
elastic-cli search -q 'level: Error'
elastic-cli search -q 'level: Error' -o json
elastic-cli search -q 'level: Error' -o '{level} {message}'
elastic-cli search -q 'level: Error' -f 'level,message'
```

For more documentation use help:
```
elastic-cli help
elastic-cli search --help
elastic-cli config --help
```
