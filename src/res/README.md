# Resources

## TL;DR

This folder needs to contain the claim certificate and the AWS CA and the serial number of the simulated device:

```text
|- AmazonRootCA1.pem
|- claim_certificate.pem
|- claim_private_key.pem
|- serial_number.txt
```

## Configuration via the AWS console

> Note: it is also possible to carry out the configuration steps using the command line.
>
> Please consult
> [Provision your device in AWS IoT Core](https://docs.aws.amazon.com/iot/latest/developerguide/iot-dc-install-provision.html#iot-dc-install-dc-provision)
> for further details.

### Claim certificates

You can generate these files, for example, in the AWS Console
under `IoT Core/Security/Certificates/Add Certificate/Create Certificate`.

Important: please make sure to download the files locally as they will be compiled into the provisioning binary as
static resources.

### IoT policy for self-provisioning

Once created (saved locally and also activated), the claim_certificate needs an activated IoT policy attached to it in
the AWS console (see `provisioning_policy_template.json` for creating this
policy using `IoT Core/Security/Policies/Create policy`).

### Provisioning template

Further, the provisioning template with the name `iot_provisionin_template` must exist in the AWS console (
see `provisioning_template.json` for creating the provisioning
template using `IoT Core/Connect/Connect many devices/Create provisioning template`)

### IoT thing policy

Furthermore, the provisioning template assumes that the devices will have an associated policy named `thing_policy` (
see `thing_policy_template.json` for creating this policy using `IoT Core/Security/Policies/Create policy`).

### Serial number

Finally, feel free to change the serial number in `serial_number.txt` but make sure that the content is a valid IoT
Thing name.
