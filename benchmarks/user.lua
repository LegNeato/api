wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"
wrk.headers["Accept"] = "application/json"
wrk.rawData = '{"query":"{\n  package(name: \"sass\") {\n    name,\n    owner,\n    normalizedName,\n    latestVersion,\n    latestStableVersion,\n    packageUploadNames\n  }\n}","variables":{}}'
