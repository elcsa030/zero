use ff::PrimeField;
use lazy_static::lazy_static;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Fr([u64; 4]);

pub const CELLS: usize = 3;
pub const ROUNDS_HALF_FULL: usize = 4;
pub const ROUNDS_PARTIAL: usize = 42;
pub const ROUNDS_TOT: usize = 2 * ROUNDS_HALF_FULL + ROUNDS_PARTIAL;

// Taken from https://extgit.iaik.tugraz.at/krypto/hadeshash
// poseidon_params_n254_t3_alpha8_M128.txt
pub const ROUND_CONSTANTS_STR: [&str; ROUNDS_TOT * CELLS] = [
    "5888606379639939820599455322177884368477873312756472112671988140021184080705",
    "18489058766968673146528183220413613399079683759169985690098361647140265017944",
    "14171222512079213058951873267355514924931036792627912326743169875439702206118",
    "7953458972600799337425196517544495617479843698017972289218559959443947182127",
    "4300918711335579235745710047083247844343576107300239508295737786313964818275",
    "4167438652994951195435584997378458884629610420320490029426218026616901799898",
    "17949040892286973664302691976853932693467507517690185711184102951328609454025",
    "20861406848526853420642932448318309420011767078588303729475682868946538144683",
    "7262411977812261428140879068003518540944833563575253826789704960794968361807",
    "12328989964685121252754163452810667637279622244169941214018587976732737183264",
    "3652962203665659181674339888095432317824203122245181162815580549114546443150",
    "2145147157573387493262407739248780416444248062379583997369283976071913961683",
    "10120851544656724975851350992037592456323453708525501772126006700747775316700",
    "11547971020749024315676854826107307606778772795504330473193921486231333256110",
    "19179018171873829442723455182222746494630468183737541368498494216335412086375",
    "9138022979701970195741680935087243342110725418927487048666023812096783434726",
    "20173930827842989450409411529023726146565196368119193993692847872167701284392",
    "9937233790609261403418692594975651335319080189368896595477373631689502510370",
    "19679671584153741404886853622796262063934212825436901241193189598360078851536",
    "12098918666190893797412141849770166820769308968069733267600812890858673048616",
    "20123117797363222287419053227673542600749202801057648393736268721557248713624",
    "17539251434666813131594923084585727066127915706934321172786695234561935777676",
    "3611673385362838953883289075082536469307875182203756336957189984625582047749",
    "4618595266012482416096816401357768820092623492588485012085922568490440184491",
    "14170794895022893458563614427743711421842824446183740079524296821842354748674",
    "3331000926299013555686223847766426831195825460795773925422101228742301019364",
    "14834146759326635667235041478267335578505874660837479557171353379212470446570",
    "17128250480450386946813539506012764742000125142654646615181919322213386479819",
    "20901366401224153643458964646186245630006408523188365016191718782486302301394",
    "15727370476292217815519337425184794449536482057644753251921603024875032616372",
    "5871090926198565537635581968384948334947983048651150003884282596647243968244",
    "12228199704791172178998389811030256533460066432463867835700976781754241350858",
    "1471721788146513878259603631267342116926752705832455346159669412087029779795",
    "21800365991052794856694014725367564059996310003876966856245855625410401089287",
    "4427639826240263782911009489247857662514845502665165375179313787487237457366",
    "10350477442582793218306695709383697050221224353435190020564862441476555509153",
    "10072792479013064979865892700375899749984984646414683329015190219171521266474",
    "4706411147430412217912922180448899883460595481508091334111760910898592920800",
    "6039924279047390441109637072672774097168891959896827383335421365291016934317",
    "3860632557233636508323974104742227857271174941836863661779309004601952382221",
    "19207643459364692436704354260635856466144153626558896589424596958234891154493",
    "7668042801799302127078984069855150173090363563852980598274469313220330021038",
    "13518808453941258090220215948855841951169618528188678496442768329964493462688",
    "12666289277657600896163671490343162325589062631535946513964579448277298690352",
    "2367232366692361907856528874742673979859818880680586479964634203173131412194",
    "16642166964583547005448771688653334469215195379813293200937659908554206153819",
    "9813612500622481175985000129750671993522705812554084791500660500148564685822",
    "19871802278095592537731075517840792431617814573326854661293991417712794882066",
    "1013437616449569496142899362414291165236375166755104588908792578514959154322",
    "8555065794108741482943736210735096050653085353764316614604960631550144993428",
    "12540679321546132653659895780671681426788751476607107967913868138968539741324",
    "9827019181587178257193040769370834193432957273138551719599488790978322592957",
    "8616625025353591127392556724642374476268339723957224199486661175525262659269",
    "1247777045148019602463339852943336019812408471693290835740068804270553088207",
    "13930885890981184996129831627604638611416847776869799582066096714660031069733",
    "15069824619991588749248397576233330368250797084352716727661771428502524730506",
    "9072442497392847059755886691007073592688314929701935644527731963812357091045",
    "14889503103762054529533438438068288808377359774920607657460256372909127013211",
    "13571931386197300103057705208272598312001235089124075622272208983466458487473",
    "7227893998616549526725900203338146158162519694853080987677416136528422987093",
    "14305172038689133883408228680805076155554013338186832423461678919334592546006",
    "1421261926431999927221892021078793730993235194012086457550693456998811821224",
    "5727671761685936011099100535815091301666887407442756275106687696083880563248",
    "11505361558978588285810793596828840547235354257011940420241035114855944530408",
    "11035821566468821387000305815692416086341728072100297454730323055488315060930",
    "2123160842041619544970391516611687539813818510627723546544219421396716110024",
    "9600601726814678137151906703407191110280457029785784892620204493406370014253",
    "3832007302669891747703712634222214296510654505694323309289115471014309371148",
    "16746291409952091089936793688601569646967242152159381048200389117266469358857",
    "460004530994075145350555176377943876664309610437171748077193448770027124308",
    "19201388326697207186723386129512741277957466359166412718066889686774086103043",
    "15283387025821459679000045617497897549024323366253578927912363831172935022735",
    "12430693467726814688381987807132400870990347894634169655609457788155787654705",
    "18191505400641819179107789287305390333565866859302846119453625719143978545127",
    "2249973315869523511373139967956003830590154169735542605379062808717650392296",
    "20946715598392977265413598111271618356845681655972782977529062585676197256470",
    "16788179626027638904601991280427772668649495300139372788586642982081062915038",
    "18940018218831720500276343704932996586797681562102147787460093020072150183135",
    "500673968830030237277409178889763590918978387125773608120094550347695994745",
    "1023463348523557754233876766422154431242831561772397419071877916499353788865",
    "15557437717377102684123424119284151235750878359539160060920905750160820169915",
    "13738754873931669846142090661903545016342206874166666225857608813801277530068",
    "6479000048605872383796271810368054637062562086687678180550561749987600434643",
    "10918082302763567551654266516708600923083170489886517851666729938507430101301",
    "10322639175815261897115243562950860337079695981773489481632004137212431891763",
    "19630761314159258491523920655999455683836439962913702591347886451564864971579",
    "21801391969430648100414543241823388822951844644866960769017161062350560473116",
    "3091956866999123084855833261993844258361624760273540844968563834076626212131",
    "9060163326566763912940630022647662028480932437101932695869068074323709583927",
    "17493147167113497684552364386512280992205869996004401701912514468177862108112",
    "21746985845614400268665193656845436168478445481518937949788420019634099867860",
    "6505583837006530541140515727010729176877960940098906364663290065588792151609",
    "4354410763820594394999160587761889004292777595816537027380103579496638434461",
    "5060380382241638634340778341198793156852929554182088975518249634793434045002",
    "15945043896201461206823417861324485110584303492552553483937511835318445600921",
    "11264861194833569761257245306321338507522139244016771927293611978602848243918",
    "13601833349141125037134662675658602354898052184858553695138851052525626266202",
    "18440582777832685470594232867441004461298338855551057142581951513049463121360",
    "16189731588904520188095006238066766120419416757059326989690304729845025318442",
    "16897383732354394438442051975484319201525443041140536117789147399160765631419",
    "20502331757841119954105478430690985178415786337477279910119483430459148869820",
    "3497090289897946494189096311641128126287921287670608412144252126478349174924",
    "13126451620559866582230467925988035892583475531801541716846741783559497646388",
    "1252264399662796843618041642884442415458419997069515567593217933485891482042",
    "20948226181117460474506440532788045938195228113724889827086508507908364242226",
    "12185898306493970655834462946276317618349110470712205566980379152501036342519",
    "16889481451509817849292967059570043384657496509212897744896152892129835321409",
    "3422927995812033370145585377552172461529744106253601015932480330907170435851",
    "3581811892034723058710373348397545631442428537653567982593876015680792908911",
    "10853720348894326406036972187342828948641427054621120124400438689235935315016",
    "2484720218763226388611721845513930218434479727797125558346895352271069336255",
    "5703717815229435885903343156503746808035891215326824058884741260327133210907",
    "8155163060937329490892104083059864801437125780231555717075798493165467932354",
    "7438014392413950730804281466505256029815502366941343110401235243089862469275",
    "10197711348440139452174647172833765538816858090200239782167416973886717843558",
    "13509016777725164101143897888222752968221344839196838843888106173129944829931",
    "7704405401320098871771383951943772478156551294808397159069327052862367571887",
    "5382672918575669305133811902911962040345877073103978949924055041444668220356",
    "16484978024521875971539367201673983973659509898069519170378541796638949331427",
    "11137007052178140769015041514129480125492900890211232839279511518525611794964",
    "7070135556492271472207130739921924793241980224909901361920728930317998573175",
    "3240730882915111397823633347985997567084982795631848736754130062574491610960",
    "8958448921662374779962910974236287499077197367258864649581376236370693497721",
    "18573648424375775266905024577482041911646210639882812974541579578109609404737",
    "19160357143605184899410908961900799146869108493081620293197250822290447130879",
    "19099902662673002519095885445811988881074690788992863399001166505083049721751",
    "12982718472690383277369121920276068669887657694740197418060909305247205035534",
    "7014468757198412384733492208684818213165322758889361181574016682757250953778",
    "770209637952045397785521394290085078756699263577456438878546727638525998576",
    "3866525023159558112079074246230900786280274896549735593844502425548303361531",
    "18349867024572241649834729035297444491263918597784734991528835298875912581729",
    "286073665149464288304193216081501915267678585324765618418269228986839279497",
    "15063399300025648305814724324901725972723611383896691972713727053638282212116",
    "2403990414306113454318790449287454943548753759680198096792957876037002535315",
    "16170580603857828773960987096130071582344267131356612147530194814294739049004",
    "9763491432211410519417668993625575804988623978864289756041428729937936735848",
    "228424857564944297115556105854640784706924299034731756162939379091964947172",
    "21773375460574706414775061912425642779009107038649141991459639150307303564820",
    "4250959397734515689478911286662101270302767511884899948084606720657861710422",
    "12685764190224445228483947388461636493190921115643710782751677852911225177991",
    "3942038592689071597271011203600516943418050149649431575064756071177159127288",
    "13454324432314049440771017134964768638545206940080049663464917647834838779133",
    "18191091206692318088756888227614434511004841134031116576279944805278422190269",
    "9718513961706510379837800593160569473677751968025789136323157665538997398908",
    "15740251704462165905585712135967223330942597062231981512626551147053857413387",
    "7359708663572757516705551248919708281744257429943695053036437407304987153393",
    "3218416338803641040878550941338662976441291365008903087707641171200361763913",
    "19903141952977967976586126222165209574476842472577647711164464099162876738236",
    "12636871384924690496414463524895177017150756447401184846852891152804508061044",
    "14533845515930494974466408792543952530064904726462188004585813120888450727620",
];

pub const MDS_STR: [&str; CELLS * CELLS] = [
    "1017023420312749934459631888016684831103661859468634146122522913463228918266",
    "4202690840311939897388485109889706482463055357734653156799117224889688752161",
    "3652883699933663193266146845129996570393662503709132891357947639536776420532",
    "7233748528128124088502224489025742672797230944400180271359847210035693908749",
    "4923546034019884502489587909962336981165992564156415634411859863408519801874",
    "5421573390662263544338021619711769416528875124357163999725140994413041443392",
    "10627066724067015316652745181519385171205666190177356149478133059870092683836",
    "5173418176910143312003315744533858560534061813808878967300644181097034998896",
    "3242101671879902378755777493421615650671562290095873775783342821028960288874",
];

lazy_static! {
    pub static ref ROUND_CONSTANTS: Vec<Fr> = {
        let mut out = Vec::<Fr>::new();
        for s in ROUND_CONSTANTS_STR {
            let as_fp = Fr::from_str_vartime(s).unwrap();
            out.push(as_fp);
        }
        out
    };
    pub static ref MDS: Vec<Fr> = {
        let mut out = Vec::<Fr>::new();
        for s in MDS_STR {
            let as_fp = Fr::from_str_vartime(s).unwrap();
            out.push(as_fp);
        }
        out
    };
}
