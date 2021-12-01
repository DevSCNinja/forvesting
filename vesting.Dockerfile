FROM projectserum/build:v0.17.0

COPY / /vesting_schedule
WORKDIR /vesting_schedule

RUN yarn
RUN ./vesting.sh