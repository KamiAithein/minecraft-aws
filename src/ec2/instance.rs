use std::default::Default;
use rusoto_core::{Region, HttpClient};
use rusoto_ec2::{TagSpecification, Tag, Instance, DescribeInstancesResult, DescribeInstancesRequest, Reservation, RunInstancesRequest, Ec2Client, Ec2, DescribeSpotInstanceRequestsRequest};
use rusoto_sts::{StsClient, StsAssumeRoleSessionCredentialsProvider};
use std::error::Error;

const AMI_TYPE:&str = "t2.micro";
const AMI_ID:&str = "ami-07efac79022b86107"; //ubuntu
const ROLE_ARN:&str = "arn:aws:iam::417217345236:role/i_am_admin";

const INSTANCE_TAG:&str = "minecraft";

pub fn region() -> Region {
    return Region::UsEast2;
}

pub fn tag(key: String, val: String) -> Tag {
    return Tag{key: Some(key.parse().unwrap()), value: Some(val.parse().unwrap())};
}
pub struct Ec2Object {
    pub client: Ec2Client,
    pub image_id: String,
    pub instance_type: String,
    tag: Tag,
    instance: Instance
}
impl Ec2Object {
    fn default_provider() -> StsAssumeRoleSessionCredentialsProvider {
        let sts = rusoto_sts::StsClient::new(region());
        StsAssumeRoleSessionCredentialsProvider::new(
            sts,
            ROLE_ARN.to_string(),
            "session-name".to_string(),
            None,
            None,
            None,
            None
        )
    }
    fn default_ec2_client() -> rusoto_ec2::Ec2Client {
        Ec2Client::new_with(HttpClient::new().unwrap(), Self::default_provider(), region())
    }
    fn instances_from_res(tag: Tag, reservation:Reservation)
                         -> Option<Vec<Instance>> {
            let mut instance_matches: Vec<Instance> = reservation.instances?
                .into_iter()
                .filter(|i| {
                    return match &i.tags {
                        Some(val) => val.contains(&tag),
                        None => false
                    };
                })
                .collect::<Vec<Instance>>();
            if !instance_matches.is_empty() {
                return Some(instance_matches);
            }
         None
    }
    pub async fn retrieve(instance_tag: &str) -> Option<Self> {
        let ec2_client = Self::default_ec2_client();
        let desc_instances_req = DescribeInstancesRequest::default();
        let desc_instances_res = match ec2_client.describe_instances(desc_instances_req).await {
            Ok(val) => val,
            Err(e) => panic!(e)
        };
        let instances_opt =
            Self::instances_from_res(tag(instance_tag.clone().to_string(), instance_tag.clone().to_string()), desc_instances_res.reservations?.remove(0));
        return match instances_opt {
            Some(mut instances) => {
                let instance = instances.remove(0);
                Some(Ec2Object {
                    client: Self::default_ec2_client(),
                    image_id: instance.image_id.as_ref()?.to_string(),
                    instance_type: instance.instance_type.as_ref()?.to_string(),
                    tag: tag(instance_tag.clone().to_string(), instance_tag.clone().to_string()),
                    instance
                })
            },
            None => {None}
        }

    }
    pub async fn new(instance_tag:&str) -> Option<Self> {
        // https://github.com/rusoto/rusoto/issues/1102
        //Thank god for Jonhoo
        let ec2_client = Self::default_ec2_client();
        let run_req = RunInstancesRequest {
            instance_type: Some(AMI_TYPE.to_string()),
            image_id: Some(AMI_ID.to_string()),
            min_count: 1,
            max_count: 1,
            tag_specifications: Some(
              vec![TagSpecification{
                  resource_type: Some("instance".to_string()),
                  tags: Some(vec![tag(instance_tag.to_string(), instance_tag.to_string())])
              }]
            ),
            ..Default::default()
        };
        return match ec2_client.run_instances(run_req).await {
          Ok(res) => Some(Ec2Object {
              client: ec2_client,
              image_id: AMI_ID.parse().unwrap(),
              instance_type: AMI_TYPE.parse().unwrap(),
              tag: tag(instance_tag.to_string(), instance_tag.to_string()),
              instance: Self::instances_from_res(tag(INSTANCE_TAG.to_string(), INSTANCE_TAG.to_string()), res)?.remove(0)
          }),
            Err(e) => panic!(e)
        }
    }
    pub async fn status(&self) -> Option<()>{
        let desc_instances_req = DescribeInstancesRequest::default();
        let desc_instances_res = match self.client.describe_instances(desc_instances_req).await {
            Ok(val) => val,
            Err(e) => panic!(e)
        };
        let reservation = desc_instances_res.reservations?.remove(0);
        let instance_opt = Self::instances_from_res(tag(INSTANCE_TAG.to_string(), INSTANCE_TAG.to_string()), reservation);
        match instance_opt {
            Some(instance) => {
                println!("{:?}", instance);
            },
            None => {panic!("Instance doesnt exist!");}
        }
        Some(())
    }
}
//TMP_VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV
//TMP_^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//CORE_VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV

//OK new -> Self
//default -> Self
//OK retrieve -> Self | retrieve already existing ec2
//status(s) -> Option<Status> | on, name,
//stop(s)
//terminate(s)
//start(s)

//CORE_^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//SSH_VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV

//get_ssh(s) -> SSH

//SSH_^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^