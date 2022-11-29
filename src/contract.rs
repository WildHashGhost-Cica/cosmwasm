#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, GetPollResponse};
use crate::state::{Config, CONFIG, Poll, POLLS};



const CONTRACT_NAME: &str = "crates.io:cosmwasm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let validated_admin_address = deps.api.addr_validate(&msg.admin_address)?;

    let config = Config {
        admin_address: validated_admin_address,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePoll { question } => execute_create_poll(deps, env, info, question),
        ExecuteMsg::Vote { question, choise } => execute_vote(deps, env, info, question, choise),
        
    }
}

fn execute_create_poll(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    question: String
) -> Result<Response, ContractError>{

    if POLLS.has(deps.storage, question.clone()){
       return Err(ContractError::CustomError { val: "Key already taken".to_string() });
    }

    let poll = Poll{question: question.clone(), yes_votes:0, no_votes:0};
    POLLS.save(deps.storage, question, &poll)?;

    Ok(Response::new().add_attribute("action", "create_poll"))
}

fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    question: String,
    choice: String,
) -> Result<Response, ContractError>{
    if !POLLS.has(deps.storage, question.clone()){
        return Err(ContractError::CustomError { val: "Poll does not exist".to_string() });
     }
    let mut poll = POLLS.load(deps.storage, question.clone())?;

    

    
    if choice != "yes" && choice != "no"{
        return Err(ContractError::CustomError { val: "Unrecognised choice".to_string() });
    } else {
        if choice == "yes" {
            poll.yes_votes += 1;
        } else {
            poll.no_votes += 1;
        }
        POLLS.save(deps.storage, question, &poll)?;

        Ok(Response::new().add_attribute("action", "vote"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoll { question } => query_get_poll(deps, env, question),
        QueryMsg::GetConfig {  } => to_binary(&CONFIG.load(deps.storage)?),
    }
}
fn query_get_poll(deps: Deps, _env: Env, question: String) -> StdResult<Binary>{
    let poll = POLLS.may_load(deps.storage, question)?;
    to_binary(&GetPollResponse{poll})
}

#[cfg(test)]
mod tests {
    
    use cosmwasm_std::{attr, from_binary, Addr};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use crate::msg::{InstantiateMsg, self, ExecuteMsg, QueryMsg, GetPollResponse};

    use super::instantiate;
    use crate::contract::{execute, query};
    use crate::state::{Poll, Config};

    

    #[test]
    fn test_instantiate(){
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr", &[]) ;
        let msg = InstantiateMsg{
            admin_address: "addr".to_string(),
        };
        //we call the instantiate function

        let resp = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(resp.attributes, vec![
            attr("action", "instantiate")
        ]);

        let msg = QueryMsg::GetConfig {  };
        let resp = query(deps.as_ref(), env, msg).unwrap();
        let config:Config = from_binary(&resp).unwrap();
        assert_eq!(config, Config{admin_address: Addr::unchecked("addr")});

    }

    #[test]
    fn test_create_poll(){
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr", &[]);
        let msg = InstantiateMsg{
            admin_address: "addr".to_string()
        };

        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg);

        let msg = ExecuteMsg::CreatePoll { 
            question: "Do you like Cosmos?".to_string()
        };

        let resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(resp.attributes, vec![
            attr("action", "create_poll")
        ]);

        let msg = QueryMsg::GetPoll { 
            question: "Do you like Cosmos?".to_string(),
        };

        let resp = query(deps.as_ref(), env.clone(), msg).unwrap();
        let get_polls_response: GetPollResponse = from_binary(&resp).unwrap();
        assert_eq!(get_polls_response, GetPollResponse{poll: Some(Poll{
            question: "Do you like Cosmos?".to_string(),
            yes_votes: 0,
            no_votes: 0,
        })});

        let msg = ExecuteMsg::CreatePoll { 
            question: "Do you like Cosmos?".to_string()
        };

        let _resp = execute(deps.as_mut(), env, info, msg).unwrap_err();
        

        
    }

    #[test]
    fn test_vote(){
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr", &[]);
        let msg = InstantiateMsg{
            admin_address: "addr".to_string()
        };

        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg);
        
        let msg = ExecuteMsg::CreatePoll { 
            question: "Do you like Cosmos?".to_string()
        };

        let _resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Vote { 
            question: "Do you like Cosmos?".to_string(), 
            choise: "yes".to_string(),
        };
        let resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(resp.attributes, vec![
            attr("action", "vote"),
        ]);

        let msg = QueryMsg::GetPoll { 
            question: "Do you like Cosmos?".to_string(),
        };

        let resp = query(deps.as_ref(), env.clone(), msg).unwrap();
        let get_polls_response: GetPollResponse = from_binary(&resp).unwrap();
        assert_eq!(get_polls_response, GetPollResponse{poll: Some(Poll{
            question: "Do you like Cosmos?".to_string(),
            yes_votes: 1,
            no_votes: 0,
        })});

        let msg = ExecuteMsg::Vote { 
            question: "Do you like ETH?".to_string(), 
            choise: "no".to_string(),
        };
        let _resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        
        let msg = ExecuteMsg::Vote { 
            question: "Do you like Cosmos?".to_string(), 
            choise: "Maybe".to_string(),
        };
        let _resp = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        
        

    }
}
