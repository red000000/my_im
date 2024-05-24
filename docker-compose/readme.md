# 注意  
kafka集群需要指定实际ip而不能用localhost/127.0.0.1，  
因为docker内部有网络，  
localhost/127.0.0.1指向容器自身ip导致consumer无法连接kafka集群，  
在这里我使用的是明文传输（KAFKA_CFG_LISTENER_SECURITY_PROTOCOL=PLAINTEXT），也可改为SSL加密传输。  

# Note  
The Kafka cluster needs to specify the actual IP address instead of localhost/127.0.0.1.  
Because docker has a network inside,  
localhost/127.0.0.1 points to the IP address of the container itself, causing the consumer to be unable to connect to the Kafka cluster.  
Here I use the plaintext transmission (KAFKA_CFG_LISTENER_SECURITY_PROTOCOL=PLAINTEXT), but you can also change to SSL encrypted transmission.  