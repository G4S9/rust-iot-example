# Rust IoT Example

## TL;DR

- IoT fleet (i.e. device self-) provisioning using a claim certificate
- since device self-provisioning is a multiple-step process, implemented state management using the redux pattern
- implemented operation phase with the obtained device certificates (simulating a temperature sensor reporting random
  values in a given f64 range)

(see also: https://github.com/G4S9/rust-iot-pre-provision-hook-example for implementing a Rust IoT pre-provisioning
hook, that can be used to further validate self-registration requests)

## Building the project

If you wish to build and run the project, some preparation is necessary, as (for obvious reasons) the claim
certificate's key (and so the certificate itself) could not be checked into the repository.

The `README.md` in the `src/res` folder has the necessary information for creating the claim private key, certificate,
provisioning policy, thing policy and provisioning template used by the project.

## Running the binaries (i.e. `provision` or `operate`)

The `IOT_HOST` and `IOT_PORT` environment variables must be set to be able to run the project's binaries.

> Hints:
>
> - the `IOT_PORT` should be set to 8883
> - the `IOT_HOST` can be obtained by running `aws iot describe-endpoint --endpoint-type iot:Data-ATS` in an AWS
    authenticated terminal

Further, for running `provision` an AWS authenticated environment is also necessary (either through `aws sso login`, AWS
user credentials or environment variables set etc.)
