use rand::prelude::*;
use std::sync::{Arc, Mutex};
use std::{
    thread::{self, sleep, JoinHandle},
    time,
};

const TIME_MULTIPLIER: u64 = 1;

enum Feedback {
    NoneYet,
    Interviewing,
    PositionOffered,
    Hired,
    Declined,
}

#[derive(Clone)]
struct Person {
    name: String,
    application_pending: Arc<Mutex<bool>>,
    feedback: Arc<Mutex<Feedback>>,
}

#[derive(Clone, Copy)]
enum ApplicationStatus {
    New,
    Considering,
    Interviewing,
    Interviewed,
    ProceedToOffer,
    OfferSent,
    OfferAccepted,
    OfferDeclined,
    Hired,
    NotAMatch,
}

#[allow(dead_code)] // todo: check resume and cover letter
struct Application {
    submitter: Person,
    resume: &'static [u8],
    cover_letter: &'static str,
    status: ApplicationStatus,
}

struct Recruiter {
    name: String,
}

impl Person {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            application_pending: Arc::new(Mutex::new(false)),
            feedback: Arc::new(Mutex::new(Feedback::NoneYet)),
        }
    }

    fn look_for_work(self, applications: Arc<Mutex<Vec<Application>>>) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                let mut application_pending = self.application_pending.lock().unwrap();
                if *application_pending {
                    let mut feedback = self.feedback.lock().unwrap();
                    match *feedback {
                        Feedback::NoneYet => println!("<{}> waiting for feedback", self.name),
                        Feedback::Interviewing => {
                            let mut applications = applications.lock().unwrap();
                            if rng.gen::<f32>() <= 0.05 {
                                println!(
                                    "<{}> I decided not to continue with this position",
                                    self.name
                                );
                                self.inform_recruiter(
                                    &mut applications,
                                    ApplicationStatus::NotAMatch,
                                );
                                *application_pending = false;
                            } else {
                                self.inform_recruiter(
                                    &mut applications,
                                    ApplicationStatus::Interviewed,
                                );
                            }
                        }
                        Feedback::PositionOffered => {
                            let mut applications = applications.lock().unwrap();
                            if rng.gen::<f32>() <= 0.05 {
                                println!("<{}> I decided not to accept the offer", self.name);
                                self.inform_recruiter(
                                    &mut applications,
                                    ApplicationStatus::OfferDeclined,
                                );
                                *application_pending = false;
                            } else {
                                println!("<{}> I decided to accept the offer", self.name);
                                self.inform_recruiter(
                                    &mut applications,
                                    ApplicationStatus::OfferAccepted,
                                );
                            }
                        }
                        Feedback::Hired => {
                            println!("<{}> Super! I'm so exited!", self.name);
                            return;
                        }
                        Feedback::Declined => {
                            println!("<{}> Oh damn! I wanted that job :(", self.name);
                            *application_pending = false;
                        }
                    }
                    *feedback = Feedback::NoneYet;
                } else {
                    println!("<{}> looking for jobs", self.name);
                    if rng.gen::<f32>() > 0.5 {
                        println!("<{}> applying", self.name);
                        let application = self.create_application();
                        {
                            let mut applications = applications.lock().unwrap();
                            applications.push(application);
                            *application_pending = true;
                        }
                    }
                }
                sleep(time::Duration::from_millis(300 * TIME_MULTIPLIER));
            }
        })
    }

    fn create_application(&self) -> Application {
        Application {
            submitter: self.clone(),
            resume: &[
                0b01100001, 0b01111001, 0b01100010, 0b01100001, 0b01100010, 0b01110100, 0b01110101,
            ],
            cover_letter: "I would love to help you save the world!",
            status: ApplicationStatus::New,
        }
    }

    fn inform_recruiter(&self, applications: &mut Vec<Application>, status: ApplicationStatus) {
        for mut a in applications.iter_mut() {
            if a.submitter.name == self.name {
                a.status = status;
            }
        }
    }
}

impl Recruiter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    fn work(self, queue: Arc<Mutex<Vec<Application>>>) -> JoinHandle<()> {
        thread::spawn(move || {
            loop {
                println!("<{}> checking applications", self.name);
                {
                    let mut queue = queue.lock().unwrap();
                    let mut processed = vec![];

                    // Note: recruiter will become the bottleneck if application pool
                    // doesn't get processed fast enough. This is a known issue and
                    // should be handled by making the rest of the process fast.
                    for (index, mut application) in queue.iter_mut().enumerate() {
                        match application.status {
                            ApplicationStatus::New => self.check(&mut application),
                            ApplicationStatus::Considering => self.call(&mut application),
                            ApplicationStatus::Interviewing => (), // todo followup on interviewers
                            ApplicationStatus::Interviewed => {
                                self.check_interview_feedback(&mut application)
                            }
                            ApplicationStatus::ProceedToOffer => self.send_offer(&mut application),
                            ApplicationStatus::OfferSent => (), // todo followup with the candidate
                            ApplicationStatus::OfferAccepted => {
                                self.mark_as_hired(&mut application);
                                self.celebrate();
                                return;
                            }
                            ApplicationStatus::OfferDeclined => {
                                println!(
                                    "<{}> {} has declined the offer",
                                    self.name, application.submitter.name
                                );
                                processed.push(index);
                            }
                            ApplicationStatus::Hired => {
                                processed.push(index);
                            }
                            ApplicationStatus::NotAMatch => {
                                processed.push(index);
                            }
                        }
                    }

                    for index in processed.iter().rev() {
                        queue.remove(*index);
                    }
                }
                sleep(time::Duration::from_millis(50 * TIME_MULTIPLIER));
            }
        })
    }

    fn check(&self, application: &mut Application) {
        println!(
            "<{}> checking application from {}",
            self.name, application.submitter.name
        );
        // simulate real life
        if skill() + luck() >= 0.8 {
            application.status = ApplicationStatus::Considering;
            println!(
                "<{}> decided to consider candidate {}",
                self.name, application.submitter.name
            );
        } else {
            application.status = ApplicationStatus::NotAMatch;
            println!(
                "<{}> decided NOT to consider candidate {}",
                self.name, application.submitter.name
            );
            *application.submitter.feedback.lock().unwrap() = Feedback::Declined;
        }
    }

    fn call(&self, application: &mut Application) {
        println!("<{}> calling {}", self.name, application.submitter.name);
        // simulate real life
        if skill() + luck() >= 0.8 {
            application.status = ApplicationStatus::Interviewing;
            *application.submitter.feedback.lock().unwrap() = Feedback::Interviewing;
            println!(
                "<{}> decided to interview candidate {}",
                self.name, application.submitter.name
            );
        } else {
            application.status = ApplicationStatus::NotAMatch;
            println!(
                "<{}> decided NOT to interview candidate {}",
                self.name, application.submitter.name
            );
            *application.submitter.feedback.lock().unwrap() = Feedback::Declined;
        }
    }

    fn check_interview_feedback(&self, application: &mut Application) {
        println!(
            "<{}> checking feedback for {}",
            self.name, application.submitter.name
        );
        // simulate real life
        // todo add some decision power to the poor interviewers
        if skill() + luck() >= 0.8 {
            application.status = ApplicationStatus::ProceedToOffer;
            println!(
                "<{}> decided to offer the position to candidate {}",
                self.name, application.submitter.name
            );
        } else {
            application.status = ApplicationStatus::NotAMatch;
            println!(
                "<{}> decided NOT to proceed to the offer with {}",
                self.name, application.submitter.name
            );
            *application.submitter.feedback.lock().unwrap() = Feedback::Declined;
        }
    }

    fn send_offer(&self, application: &mut Application) {
        println!(
            "<{}> sending an offer to {}",
            self.name, application.submitter.name
        );
        application.status = ApplicationStatus::OfferSent;
        *application.submitter.feedback.lock().unwrap() = Feedback::PositionOffered;
    }

    fn mark_as_hired(&self, application: &mut Application) {
        println!(
            "<{}> {} has accepted the position!",
            self.name, application.submitter.name
        );
        application.status = ApplicationStatus::Hired;
        *application.submitter.feedback.lock().unwrap() = Feedback::Hired;
    }

    fn celebrate(&self) {
        println!("<{}> Wohoo! My work here is done! ;)", self.name);
    }
}

fn main() {
    let applications = vec![];
    let shared_applications = Arc::new(Mutex::new(applications));

    let ilari = Person::new("Ilari");
    let recruiter_working = Recruiter::new("Recruiter").work(shared_applications.clone());
    let ilari_applying_for_work = ilari.look_for_work(shared_applications.clone());

    recruiter_working.join().ok();
    ilari_applying_for_work.join().ok();
}

fn skill() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.75, 0.95)
}

fn luck() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0, 0.05)
}
