searchState.loadedDescShard("podcast_crawler", 0, "Sets a configuration value from an environment variable\nSets a string configuration value from an environment …\nValidates a configuration condition\nCrawler module for fetching and processing podcast data.\nInfrastructure layer for the podcast crawler.\nA macro for error handling with debug level logging.\nA macro for error handling with automatic logging at the …\nA macro for retrying asynchronous operations with …\nA macro for error handling with warning level logging.\nResult of a crawling task\nGet the duration of the task\nDuration of the crawl\nError message if unsuccessful\nCreate a new failed task result\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConvert the task result into a domain result\nCheck if the task was successful\nParsed data if successful\nCreate a new successful task result\nWhether the crawl was successful\nGet the URL that was crawled\nURL that was crawled\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nRSS 解析上下文，用于错误处理和状态跟踪\nParser configuration\nParsing states\nRSS feed parser\nParser state during RSS processing\n是否允许空的必需字段\nClean HTML content\n是否清理 HTML 内容\nDebugging macro for parser events.\nXML 元素路径，用于错误定位\nExtract tag name and attributes from a BytesStart event\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nParse boolean value from string\nParse date string to DateTime\n严格模式\nURL\nValidate URL format\n是否验证 URLs\n抓取单个URL的内容\n抓取并解析内容\n获取最大并发数\n解析内容\n解析feed内容为目标类型\nCalculate the similarity between two URLs based on their …\nDistribute URLs across threads to minimize similarity …\nInternal Distributor structure\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\n获取内容\n解析feed内容为目标类型\nRSS 解析上下文，用于错误处理和状态跟踪\nParser configuration\nParsing states\nRSS feed parser\n是否允许空的必需字段\nClean HTML content\n是否清理 HTML 内容\nDebugging macro for parser events.\nXML 元素路径，用于错误定位\nExtract tag name and attributes from a BytesStart event\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nParse boolean value from string\nParse date string to DateTime\n严格模式\nURL\nValidate URL format\n是否验证 URLs\nRSS爬虫系统入口\n添加新的爬取任务\nReturns the argument unchanged.\n获取所有任务状态\nCalls <code>U::from(self)</code>.\n创建新的RSS爬虫实例\n优雅关闭爬虫系统\n带超时的优雅关闭\n启动爬虫系统\n等待所有任务完成\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nPublic-facing TaskManagementSystem structure\nAdd a new task\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nGracefully shut down the system\nGracefully shut down the system with a custom timeout\nWait for all tasks to complete and return the list of tasks\nWait for all tasks to complete with a custom timeout\nInternal ThreadManager structure\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nInternal Worker structure\nWorker状态\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConfiguration management for the podcast crawler.\nError handling for the podcast crawler application.\nApplication initialization and state management.\nInitialize the application infrastructure with default …\nInitialize the application infrastructure with custom …\nLogging infrastructure for the podcast crawler application.\nApplication configuration settings\nCrawler configuration settings.\nDatabase configuration settings.\nGets the maximum number of database connections\nGets the database connection URL\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nGets the log level\nLogging configuration settings.\nConfiguration macros.\nCreates a new Settings instance\nServer configuration settings.\nGets the server address\nConfiguration utility functions.\nValidates the configuration settings\nCrawler configuration\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSets configuration values from environment variables\nValidates the crawler configuration\nDatabase configuration\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSets configuration values from environment variables\nValidates the database configuration\nLogging configuration\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSets configuration values from environment variables\nValidates the logging configuration\nServer configuration\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nSets configuration values from environment variables\nValidates the server configuration\nGets a string value from an environment variable\nGets a string value from an environment variable with test …\nParses an environment variable into a specified type\nApplication-wide error type that encompasses all possible …\nError handling trait\nContains the error value\nContains the success value\nGets the error context\nDomain-specific error types and handling.\nGets the error code\nExternal service error types and handling.\nReturns the argument unchanged.\nConverts a reqwest error into an AppError\nConverts an IO error into an AppError\nConverts a connection pool error into an AppError\nConverts a Diesel error into an AppError\nInfrastructure-related error types and handling.\nCalls <code>U::from(self)</code>.\nChecks if the error is retryable\nLogs an error if present\nNetwork-related error types and handling.\nParsing-related error types and handling.\nGets the recommended retry delay\nSets the error context\nAdds context to an error\nBatch processing errors\nDomain error type\nTypes of domain errors\nInvalid entity state errors\nEntity not found errors\nOther domain-specific errors\nUnexpected errors\nData validation errors\nGets the error code for this error\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks if this error is retryable\nCreates a new domain error\nAuthentication failures (e.g., invalid credentials)\nAuthorization failures (e.g., insufficient permissions)\nExternal error type\nTypes of external service errors\nOther external service errors\nService unavailability (e.g., maintenance, outage)\nGets the error code for this error\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks if this error is retryable\nCreates a new external error\nCache operation errors\nConfiguration loading and validation errors\nDatabase-related errors (connection, query, etc.)\nFile system and I/O errors\nInfrastructure error type\nTypes of infrastructure errors\nOther infrastructure errors\nGets the error code for this error\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks if this error is retryable\nCreates a new infrastructure error\nConnection establishment errors\nInvalid or malformed response\nNetwork error type\nTypes of network errors\nOther network-related errors\nRate limit exceeded\nRequest timeout errors\nToo many redirects in request chain\nGets the error code for this error\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks if this error is retryable\nCreates a new network error\nInvalid Atom feed format\nInvalid data format\nInvalid RSS feed format\nInvalid XML document structure\nRequired field is missing\nOther parsing-related errors\nParse error type\nTypes of parsing errors\nGets the error code for this error\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks if this error is retryable\nCreates a new parse error\nApplication repositories container\nApplication state containing all initialized components\nReturns the argument unchanged.\nReturns the argument unchanged.\nChecks if all components are healthy\nInitialize the application state with default settings\nInitialize the application state with provided settings\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreates a new instance of application repositories\nInitialize the logging system with the provided …\nDatabase infrastructure for the podcast crawler.\nDatabase context for managing database connections\nReturns the argument unchanged.\nGets a connection from the pool\nCalls <code>U::from(self)</code>.\nCreates a new <code>DatabaseContext</code> with default configuration\nCreates a new <code>DatabaseContext</code> with the provided …\nGets the underlying connection pool\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nPodcastRank 数据结构\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nHelper type for representing a boxed query from this table\nThe SQL type of all of the columns on this table\nA tuple of all of the columns on this table\nContains all of the columns of this table\nThe distinct clause of the query\nRe-exports all of the columns of this table, as well as the\nReturns the argument unchanged.\nThe from clause of the query\nThe group by clause of the query\nThe having clause of the query\nCalls <code>U::from(self)</code>.\nThe combined limit/offset clause of the query\nThe order clause of the query\nThe select clause of the query\nRepresents <code>table_name.*</code>, which is sometimes necessary for …\nThe actual table struct\nThe where clause of the query\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nRepresents <code>table_name.*</code>, which is sometimes needed for …\nHelper type for representing a boxed query from this table\nThe SQL type of all of the columns on this table\nA tuple of all of the columns on this table\nContains all of the columns of this table\nThe distinct clause of the query\nRe-exports all of the columns of this table, as well as the\nReturns the argument unchanged.\nThe from clause of the query\nThe group by clause of the query\nThe having clause of the query\nCalls <code>U::from(self)</code>.\nThe combined limit/offset clause of the query\nThe order clause of the query\nThe select clause of the query\nRepresents <code>table_name.*</code>, which is sometimes necessary for …\nThe actual table struct\nThe where clause of the query\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nRepresents <code>table_name.*</code>, which is sometimes needed for …\nHelper type for representing a boxed query from this table\nThe SQL type of all of the columns on this table\nA tuple of all of the columns on this table\nContains all of the columns of this table\nThe distinct clause of the query\nRe-exports all of the columns of this table, as well as the\nReturns the argument unchanged.\nThe from clause of the query\nThe group by clause of the query\nThe having clause of the query\nCalls <code>U::from(self)</code>.\nThe combined limit/offset clause of the query\nThe order clause of the query\nThe select clause of the query\nRepresents <code>table_name.*</code>, which is sometimes necessary for …\nThe actual table struct\nThe where clause of the query\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nRepresents <code>table_name.*</code>, which is sometimes needed for …\nHelper type for representing a boxed query from this table\nThe SQL type of all of the columns on this table\nA tuple of all of the columns on this table\nContains all of the columns of this table\nThe distinct clause of the query\nRe-exports all of the columns of this table, as well as the\nReturns the argument unchanged.\nThe from clause of the query\nThe group by clause of the query\nThe having clause of the query\nCalls <code>U::from(self)</code>.\nThe combined limit/offset clause of the query\nThe order clause of the query\nThe select clause of the query\nRepresents <code>table_name.*</code>, which is sometimes necessary for …\nThe actual table struct\nThe where clause of the query\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nRepresents <code>table_name.*</code>, which is sometimes needed for …")