# Api manager service backend

Serving API Gateway manager administrative pages

## Building process

The package is built by multi-stage Dockerfile (https://docs.docker.com/build/building/multi-stage/)

Stages and produced code are:

1. build react application: it produce index.html, index.bundle.js, index.bundle.js.map
2. build the rust executable: it produce apimanager-service executable
3. deploy executable in a scratch debian image with assets

The rust build process expects ENV variable specifying the path of static files to be loaded and served in the path, i.e.:

> ASSETS=/assets cargo build --release

## Entrypoint list

```
POST /api/login (was /authenticate/login)
payload: {"credentials":{"login":"daniele@smartango.com","password":"s4r3bb3Ora?!"}}
```

```
GET /api/list-services
response:
{
    "error": null,
    "data": {
        "services": [
            {
                "name": "accounting",
                "lastUpdate": "2025-10-16T10:53:24.576149Z",
                "lastStart": "2025-10-16T10:53:31.379571827Z"
            },
            {
                "name": "adminAPI",
                "lastUpdate": "2025-02-13T15:56:39.834671Z",
                "lastStart": "2025-02-19T09:27:45.207981867Z"
            },
            ...
        ]
    }
}
```

```
GET /api/list-resources
response:
{
    "service_images": {
        "muapi2.starsellersworld.com:446/sswbaservice2:latest": {
            "labels": {
                "com.starsellersworld.muapi": "baseservice2",
                "com.starsellersworld.name": "baseservice2",
                "com.starsellersworld.port": "6188",
                "com.starsellersworld.volumes": "['/etc/ssw_service','/common/etc/zservices', '/upload_pool']"
            },
            "name": "baseservice2"
        },
        "muapi2.starsellersworld.com:446/sswbaservice2:v0.5": {
            "labels": {
                "com.starsellersworld.muapi": "baseservice2",
                "com.starsellersworld.name": "baseservice2",
                "com.starsellersworld.port": "6188",
                "com.starsellersworld.volumes": "['/etc/ssw_service','/common/etc/zservices', '/upload_pool']"
            },
            "name": "baseservice2"
        },
        "muapi2.starsellersworld.com:446/sswmuservices/gateservice:v0.12.5": {
            "labels": {
                "com.starsellersworld.configs": "{'jwtkey.public':'/keypair/rsa_tk.pub'}",
                "com.starsellersworld.environment": "['HSQL249=http://hsqlresponder_hsql249:8080','REDIS_CACHE=redis_cache', 'REDISURL=redis://redis_cache:6379']",
                "com.starsellersworld.muapi": "baseservice2",
                "com.starsellersworld.name": "baseservice2",
                "com.starsellersworld.port": "6188",
                "com.starsellersworld.volumes": "['/etc/ssw_service','/upload_pool']"
            },
            "name": "baseservice2"
        },
        ...
    },
    "services": {
        "error": null,
        "data": {
            "zmqservices": [
                {
                    "name": "userinfoservice_userinfoservice",
                    "format": "",
                    "ztype": "rep",
                    "zport": "5553",
                    "address": "tcp://userinfoservice_userinfoservice:5553"
                },
                {
                    "name": "proceduremaster_proceduremaster",
                    "format": "http",
                    "ztype": "http",
                    "zport": "8080",
                    "address": "http://proceduremaster_proceduremaster:8080"
                },
                {
                    "name": "zcmdexecutor_zcmdphp7",
                    "format": "shell",
                    "ztype": "rep",
                    "zport": "5553",
                    "address": "tcp://zcmdexecutor_zcmdphp7:5553"
                },
                ...
            ]
        }
    },
    "permissions": [
        "admin",
        "impersonate",
        "worker",
        "ssw_customer",
        "ssw_customer_plus",
        "ssw_secondary",
        "ssw_seller",
        "ssw_vendor",
        "status_authentic",
        "status_active",
        "status_suspend"
    ]
}
```

```
GET /api/service/:servicename
response:
{
    "error": null,
    "data": {
        "definition": {
            "cpus_limit": "0.1",
            "entry_points": [
                {
                    "authtype": "JWT",
                    "jwt_perms": [
                        "admin"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "command": "echo 'php /home/web123/cjob/navision_xml/xwave_accounting_10002_.php ${url.contract} ${url.amount} ${url.qty} \"${url.purpose}\" >> tobill.log' >>  /storage/saas/CENTRAL_LOG/tobill.sh",
                            "container": "PHP7",
                            "onError": "jump_report",
                            "service": "tcp://zcmdexecutor_zcmdphp7:5553",
                            "type": "zshell"
                        }
                    ],
                    "type": "sync",
                    "url": "/putInBill/:contract/:amount/:qty/:purpose"
                },
                {
                    "authtype": "JWT",
                    "contentType": "application/json",
                    "jwt_perms": [
                        "admin",
                        "ssw_customer"
                    ],
                    "method": "POST",
                    "pipeline_steps": [
                        {
                            "container": "PHP7",
                            "onError": "jump_report",
                            "query": "SELECT ssw_id, tariff_id  FROM usr_web123_10.ssw_user_has_tariff WHERE ssw_id = ${body.ssw_id} AND main_ssw_id = ${claim.sswid}",
                            "service": "http://hsqlresponder_hsql249:8080",
                            "type": "hquery"
                        },
                        {
                            "container": "PHP7",
                            "onError": "jump_report",
                            "query": "SELECT  ssw_id, s.service_id, case when (remaining-${body.qty})>0 then 0 when (remaining-${body.qty})<=0 then service_price else credits end as cent  FROM  usr_web123_10.ssw_services as s left join  usr_web123_10.ssw_user_has_service as s1 on s1.service_id=s.service_id and ssw_id=${step0[0].ssw_id} WHERE s.service_id=${url.serviceid}",
                            "service": "http://hsqlresponder_hsql249:8080",
                            "type": "hquery"
                        },
                        {
                            "container": "PHP7",
                            "onError": "jump_report",
                            "query": "insert into usr_web123_10.service_accounting (ssw_id, service_id, amount, qty, business_id, tariff_id, accounting_date)  select ${step0[0].ssw_id}, ${url.serviceid}, ${step1[0].cent}, ${body.qty},${body.business_id},${step0[0].tariff_id},NOW() from (select 1) as c where  ${step1[0].cent}>0  and ${step0[0].ssw_id}>0  and ${url.serviceid}>0",
                            "service": "http://hsqlresponder_hsql249:8080",
                            "type": "hquery"
                        },
                        {
                            "container": "PHP7",
                            "onError": "jump_report",
                            "query": "UPDATE usr_web123_10.main_users_balance SET balance=balance-${step1[0].cent}*${body.qty}  where ${step1[0].cent}>0 and main_ssw_id=${claim.sswid} and  ${url.serviceid}>0 and ${step0[0].ssw_id}>0 ",
                            "service": "http://hsqlresponder_hsql249:8080",
                            "type": "hquery"
                        }
                    ],
                    "type": "sync",
                    "url": "/service/:serviceid"
                },
                {
                    "authtype": "JWT",
                    "jwt_perms": [
                        "ssw_customer"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "cmd": "/protocol_pdf_bookings",
                            "container": "PHP7",
                            "data": "{\"sswid\": \"${claim.sswid}\"}",
                            "service": "tcp://accounting-service_accounting-service:5553",
                            "type": "zmq_service_call"
                        }
                    ],
                    "type": "sync",
                    "url": "/protocol_pdf_bookings"
                },
                {
                    "authtype": "JWT",
                    "jwt_perms": [
                        "ssw_customer"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "cmd": "/protocol_pdf_booking_get",
                            "container": "PHP7",
                            "data": "{\"sswid\": \"${claim.sswid}\", \"id\": \"${url.id}\"}",
                            "service": "tcp://accounting-service_accounting-service:5553",
                            "type": "zmq_service_call"
                        }
                    ],
                    "type": "sync",
                    "url": "/protocol_pdf_booking_get/:id"
                },
                {
                    "authtype": "JWT",
                    "contentType": "application/json",
                    "jwt_perms": [
                        "ssw_customer"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "cmd": "/payment-documents",
                            "container": "PHP7",
                            "data": "{\"sswid\":\"${claim.sswid}\"}",
                            "service": "tcp://accounting-service_accounting-service:5553",
                            "type": "zmq_service_call"
                        }
                    ],
                    "type": "sync",
                    "url": "/payments"
                },
                {
                    "authtype": "JWT",
                    "jwt_perms": [
                        "ssw_customer"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "cmd": "/payment-document",
                            "container": "PHP7",
                            "data": "{\"sswid\":\"${claim.sswid}\", \"docid\": \"${url.docid}\"}",
                            "service": "tcp://accounting-service_accounting-service:5553",
                            "type": "zmq_service_call"
                        }
                    ],
                    "type": "sync",
                    "url": "/payment/:docid"
                },
                {
                    "authtype": "JWT",
                    "jwt_perms": [
                        "ssw_customer"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "cmd": "/payment-getstatus",
                            "container": "PHP7",
                            "data": "{\"sswid\":\"${claim.sswid}\"}",
                            "service": "tcp://accounting-service_accounting-service:5553",
                            "type": "zmq_service_call"
                        }
                    ],
                    "type": "sync",
                    "url": "/balance"
                },
                {
                    "authtype": "JWT",
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "cmd": "/payment-document-pending",
                            "container": "PHP7",
                            "data": "{\"sswid\": \"${claim.sswid}\"}",
                            "service": "tcp://accounting-service_accounting-service:5553",
                            "type": "zmq_service_call"
                        }
                    ],
                    "type": "sync",
                    "url": "/payment-pending"
                },
                {
                    "authtype": "JWT",
                    "debug": true,
                    "jwt_perms": [
                        "ssw_customer"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "cmd": "/payment-document-pdf",
                            "container": "PHP7",
                            "data": "{\"sswid\": \"${claim.sswid}\", \"id\": \"${url.id}\"}",
                            "debug": true,
                            "service": "tcp://accounting-service_accounting-service:5553",
                            "type": "zmq_service_call"
                        }
                    ],
                    "type": "sync",
                    "url": "/payment-pdf/:id"
                },
                {
                    "authtype": "JWT",
                    "debug": false,
                    "jwt_perms": [
                        "ssw_customer"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "cmd": "/payment-report-document-pdf",
                            "container": "PHP7",
                            "data": "{\"sswid\": \"${claim.sswid}\", \"business_id\": \"${url.businessid}\"}",
                            "debug": true,
                            "service": "tcp://accounting-service_accounting-service:5553",
                            "type": "zmq_service_call"
                        }
                    ],
                    "type": "sync",
                    "url": "/supportdoc/:businessid"
                },
                {
                    "authtype": "JWT",
                    "jwt_perms": [
                        "ssw_customer"
                    ],
                    "method": "GET",
                    "pipeline_steps": [
                        {
                            "container": "PHP7",
                            "query": "UPDATE usr_web123_10.main_users_balance SET current_tariff_id=(SELECT tariff_id  FROM  usr_web123_10.tariff WHERE `tariff_id` = ${url.tarif_id}) WHERE main_ssw_id=${claim.sswid}",
                            "service": "http://hsqlresponder_hsql249:8080",
                            "type": "hquery"
                        },
                        {
                            "container": "PHP7",
                            "query": "UPDATE usr_web123_10.ssw_user_has_tariff as t inner join usr_web123_10.main_users_balance as m on t.main_ssw_id=m.main_ssw_id SET t.tariff_id=m.current_tariff_id where t.main_ssw_id=${claim.ssw_id}\n",
                            "service": "http://hsqlresponder_hsql249:8080",
                            "type": "hquery"
                        },
                        {
                            "container": "PHP7",
                            "query": "UPDATE usr_web123_10.ssw_user_has_tariff as t inner join usr_web123_10.ssw_user_has_options as s on s.ssw_id=t.ssw_id SET s.tariff_id=t.tariff_id where t. main_ssw_id=${claim.ssw_id}",
                            "service": "http://hsqlresponder_hsql249:8080",
                            "type": "hquery"
                        }
                    ],
                    "type": "sync",
                    "url": "/tarif/:tarif_id"
                }
            ],
            "image": "muapi2.starsellersworld.com:446/sswmuservices/gateservice:v0.12.2",
            "memory_limit": "99M",
            "name": "accounting",
            "replicas": "1",
            "resources": [],
            "volumes": [
                {
                    "Destination": "/common/etc/dbconn",
                    "Source": "/storage/saas/docker_swarm/common/etc/dbconn"
                },
                {
                    "Destination": "/var/run/docker.sock",
                    "Source": "/var/run/docker.sock"
                }
            ]
        }
    }
}
```

Then

```
POST /api/service/:servicename
payload:
<definition data_type>
```

## Dependencies

This service depends on:

**PERMISERVICE**:
- Env: PERMISERVICE=
- Protocol: HTTP
- Entrypoints: /permissions (list of permissions)

**GEARMANSMS**:
- ENV: GEARMANSMS
- Protocol: HTTP
- Entrypoints:
  - PATCH /start/:servicename
  - PATCH /stop/:servicename
  - POST /create/:servicename <[service definition]>
  - GET /servicelist

## TODO

- serving static pages - OK
- serving api staff - OK
- attach utility service: service-manager-service - OK
- accept ENV execution variable - OK
- create docker image